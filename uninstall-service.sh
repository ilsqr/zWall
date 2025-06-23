#!/bin/bash

# zWall Service Uninstaller Script

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

echo "🗑️  zWall Service Uninstaller"
echo "============================="

# SystemD kontrolü
if [ -f "$SYSTEMD_SERVICE" ]; then
    echo "🔧 SystemD servisi kaldırılıyor..."
    
    # Servisi durdur ve devre dışı bırak
    systemctl stop zwall 2>/dev/null || true
    systemctl disable zwall 2>/dev/null || true
    
    # Service dosyasını sil
    rm -f "$SYSTEMD_SERVICE"
    
    # SystemD'yi yenile
    systemctl daemon-reload
    
    echo "✅ SystemD servisi kaldırıldı!"
fi

# OpenRC kontrolü
if [ -f "$OPENRC_SERVICE" ]; then
    echo "🔧 OpenRC servisi kaldırılıyor..."
    
    # Servisi durdur ve devre dışı bırak
    rc-service zwall stop 2>/dev/null || true
    rc-update del zwall default 2>/dev/null || true
    
    # Service dosyalarını sil
    rm -f "$OPENRC_SERVICE"
    rm -f "$OPENRC_CONFIG"
    
    echo "✅ OpenRC servisi kaldırıldı!"
fi

# Binary'yi sil
if [ -f "$BINARY_PATH" ]; then
    echo "📁 Binary kaldırılıyor..."
    rm -f "$BINARY_PATH"
    echo "✅ Binary kaldırıldı!"
fi

echo ""
echo "🎉 zWall tamamen kaldırıldı!"
