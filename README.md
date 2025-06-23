# zWall - eBPF Ping Blocker & SSH DDoS Protection

Rust ile yazılmış, eBPF tabanlı kernel seviyesinde çalışan ağ güvenlik aracı.

## Özellikler

- 🚫 **Ping Engelleme**: Tüm ICMP Echo Request paketlerini engeller
- 🛡️ **SSH DDoS Koruması**: SSH portuna (22) yönelik aşırı bağlantı girişimlerini tespit eder
- 🔒 **Otomatik IP Yasaklama**: Eşik değeri aşan IP'leri otomatik yasaklar
- ⚡ **Yüksek Performans**: XDP ile kernel seviyesinde çalışır
- 📊 **Gerçek Zamanlı İstatistikler**: Engellenen IP'ler ve SSH girişimleri görüntüler

## Gereksinimler

- Linux (Kernel 4.19+)
- Rust nightly
- Root yetkileri
- eBPF desteği

## Kurulum

### Linux/WSL
```bash
chmod +x build.sh
./build.sh
```

## Sistem Servisi Kurulumu

### Otomatik Kurulum (Linux)
```bash
# Projeyi derle
./build.sh

# Servisi kur (SystemD veya OpenRC otomatik algılanır)
sudo chmod +x install-service.sh
sudo ./install-service.sh
```

### Manuel SystemD Kurulumu (Ubuntu/Debian/CentOS)
```bash
# Binary'yi sistem dizinine kopyala
sudo cp ./target/release/ping-blocker /usr/local/bin/

# Service dosyasını kopyala
sudo cp ./systemd/zwall.service /etc/systemd/system/

# Servisi etkinleştir
sudo systemctl daemon-reload
sudo systemctl enable zwall
sudo systemctl start zwall

# Durumu kontrol et
sudo systemctl status zwall
```

### Manuel OpenRC Kurulumu (Alpine/Gentoo)
```bash
# Binary'yi sistem dizinine kopyala
sudo cp ./target/release/ping-blocker /usr/local/bin/

# Service dosyalarını kopyala
sudo cp ./openrc/zwall /etc/init.d/
sudo cp ./openrc/conf.d/zwall /etc/conf.d/
sudo chmod +x /etc/init.d/zwall

# Servisi etkinleştir
sudo rc-update add zwall default
sudo rc-service zwall start

# Durumu kontrol et
sudo rc-service zwall status
```

## Servis Yönetimi

### SystemD Komutları
```bash
sudo systemctl start zwall      # Servisi başlat
sudo systemctl stop zwall       # Servisi durdur
sudo systemctl restart zwall    # Servisi yeniden başlat
sudo systemctl status zwall     # Durum kontrolü
sudo systemctl enable zwall     # Otomatik başlatmayı etkinleştir
sudo systemctl disable zwall    # Otomatik başlatmayı devre dışı bırak
journalctl -u zwall -f          # Logları takip et
```

### OpenRC Komutları
```bash
sudo rc-service zwall start     # Servisi başlat
sudo rc-service zwall stop      # Servisi durdur
sudo rc-service zwall restart   # Servisi yeniden başlat
sudo rc-service zwall status    # Durum kontrolü
sudo rc-update add zwall default    # Otomatik başlatmayı etkinleştir
sudo rc-update del zwall default    # Otomatik başlatmayı devre dışı bırak
```

## Konfigürasyon

### SystemD Konfigürasyonu
Service dosyasını düzenleyin: `/etc/systemd/system/zwall.service`
```ini
ExecStart=/usr/local/bin/ping-blocker --iface wlan0 --threshold 5 --verbose
```

### OpenRC Konfigürasyonu
Konfigürasyon dosyasını düzenleyin: `/etc/conf.d/zwall`
```bash
ZWALL_INTERFACE="wlan0"
ZWALL_THRESHOLD="5"
ZWALL_EXTRA_ARGS="--verbose"
```

## Servis Kaldırma

### Otomatik Kaldırma
```bash
sudo chmod +x uninstall-service.sh
sudo ./uninstall-service.sh
```

## Kullanım

```bash
# Varsayılan ayarlarla çalıştır (eth0, eşik: 10)
sudo ./target/release/ping-blocker

# Özel arayüz ve eşik ile
sudo ./target/release/ping-blocker --iface wlan0 --threshold 5

# Detaylı çıktı ile
sudo ./target/release/ping-blocker --verbose
```

### Parametreler

- `--iface`: Ağ arayüzü (varsayılan: eth0)
- `--threshold`: SSH bağlantı eşiği (varsayılan: 10)
- `--verbose`: Detaylı log çıktısı

## Test Etme

### Ping Testi
```bash
# Başka bir makineden
ping <server_ip>
# Yanıt gelmeyecek
```

### SSH DDoS Testi
```bash
# SSH bağlantı saldırısı simülasyonu
for i in {1..15}; do ssh user@<server_ip> & done
# 10. bağlantıdan sonra IP yasaklanacak
```

### Trafik İzleme
```bash
# ICMP ve SSH trafiğini izle
sudo tcpdump -i eth0 icmp or port 22
```

## Proje Yapısı

```
zWall/
├── ping-blocker/           # Kullanıcı alanı program
│   ├── src/main.rs
│   └── Cargo.toml
├── ping-blocker-ebpf/      # eBPF kernel modülü
│   ├── src/main.rs
│   └── Cargo.toml
├── systemd/                # SystemD service dosyaları
│   └── zwall.service
├── openrc/                 # OpenRC service dosyaları
│   ├── zwall
│   └── conf.d/zwall
├── .cargo/config.toml      # Cargo yapılandırması
├── build.sh               # Linux build scripti
├── build.ps1              # Windows build scripti
├── install-service.sh     # Sistem servisi kurulum scripti
├── uninstall-service.sh   # Sistem servisi kaldırma scripti
└── README.md
```

## Güvenlik Notları

- Program root yetkisi gerektirir
- Yanlış konfigürasyon ağ erişimini engelleyebilir
- Test ortamında deneyip production'a geçin
- SSH eşiğini çok düşük ayarlamayın (meşru kullanıcıları engelleyebilir)

## Kaldırma

Program Ctrl+C ile durdurulabilir. XDP programı otomatik olarak ağ arayüzünden kaldırılır.

## Lisans

GPL v2 - eBPF programları GPL lisansı gerektirir.
