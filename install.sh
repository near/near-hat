
#!/bin/bash

brew install dnsmasq mitmproxy

mkdir -pv $(brew --prefix)/etc/
echo 'address=/.nearhat/127.0.0.1' >> $(brew --prefix)/etc/dnsmasq.conf
sudo mkdir -v /etc/resolver
sudo bash -c 'echo "nameserver 127.0.0.1" > /etc/resolver/nearhat'