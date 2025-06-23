#!/bin/bash

# zWall Service Installation Script

set -e

# Değişkenler
BINARY_PATH="/usr/local/bin/ping-blocker"
SYSTEMD_SERVICE="/etc/systemd/system/zwall.service"
OPENRC_SERVICE="/etc/init.d/zwall"
OPENRC_CONFIG="/etc/conf.d/zwall"

# Root kontrolü
if [[ $EUID -ne 0 ]]; then
   echo "Bu script root olarak çalıştırılmalıdır" 
   exit 1
fi

echo "🚀 zWall Service Installer"
echo "=========================="

# Binary'nin varlığını kontrol et
if [ ! -f "./target/release/ping-blocker" ]; then
    echo "❌ Binary bulunamadı. Önce projeyi derleyin:"
    echo "   ./build.sh"
    exit 1
fi

# Binary'yi sistem dizinine kopyala
echo "📁 Binary kopyalanıyor..."
cp "./target/release/ping-blocker" "$BINARY_PATH"
chmod +x "$BINARY_PATH"

# Systemd mi OpenRC mi kontrol et
if systemctl --version >/dev/null 2>&1; then
    echo "🔧 SystemD servisi kuruluyor..."
    
    # Service dosyasını kopyala
    cp "./systemd/zwall.service" "$SYSTEMD_SERVICE"
    
    # SystemD'yi yenile
    systemctl daemon-reload
    
    echo "✅ SystemD servisi kuruldu!"
    echo ""
    echo "Kullanım:"
    echo "  sudo systemctl enable zwall    # Sistem başlangıcında otomatik başlat"
    echo "  sudo systemctl start zwall     # Servisi başlat"
    echo "  sudo systemctl stop zwall      # Servisi durdur"
    echo "  sudo systemctl status zwall    # Servis durumunu kontrol et"
    echo "  journalctl -u zwall -f         # Logları takip et"
    
elif rc-service --exists >/dev/null 2>&1; then
    echo "🔧 OpenRC servisi kuruluyor..."
    
    # Service dosyalarını kopyala
    cp "./openrc/zwall" "$OPENRC_SERVICE"
    cp "./openrc/conf.d/zwall" "$OPENRC_CONFIG"
    
    # İzinleri ayarla
    chmod +x "$OPENRC_SERVICE"
    chmod 644 "$OPENRC_CONFIG"
    
    echo "✅ OpenRC servisi kuruldu!"
    echo ""
    echo "Kullanım:"
    echo "  sudo rc-update add zwall default   # Sistem başlangıcında otomatik başlat"
    echo "  sudo rc-service zwall start        # Servisi başlat"
    echo "  sudo rc-service zwall stop         # Servisi durdur"
    echo "  sudo rc-service zwall status       # Servis durumunu kontrol et"
    echo ""
    echo "Konfigürasyon: /etc/conf.d/zwall"
    
else
    echo "❌ Ne SystemD ne de OpenRC bulunamadı!"
    echo "Manuel olarak çalıştırın: sudo $BINARY_PATH --iface eth0 --threshold 10"
    exit 1
fi

echo ""
echo "⚠️  Önemli Notlar:"
echo "   - Servis root yetkisi ile çalışır"
echo "   - Varsayılan ayarlar: eth0 arayüzü, 10 SSH eşiği"
echo "   - Service dosyalarını ihtiyacınıza göre düzenleyin"
