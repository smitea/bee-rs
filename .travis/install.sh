sudo mkdir /var/run/sshd
sudo echo 'root:admin' |chpasswd
sudo sed -ri 's/^#?PermitRootLogin\s+.*/PermitRootLogin yes/' /etc/ssh/sshd_config
sudo sed -ri 's/UsePAM yes/#UsePAM yes/g' /etc/ssh/sshd_config
sudo mkdir /root/.ssh 
sudo apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
sudo /etc/init.d/ssh restart