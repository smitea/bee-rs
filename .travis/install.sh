mkdir /var/run/sshd
echo 'root:root' |chpasswd
sed -ri 's/^#?PermitRootLogin\s+.*/PermitRootLogin yes/' /etc/ssh/sshd_config
sed -ri 's/UsePAM yes/#UsePAM yes/g' /etc/ssh/sshd_config
mkdir /root/.ssh
apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
/usr/sbin/sshd -D