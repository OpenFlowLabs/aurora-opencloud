# -*- mode: ruby -*-
# vi: set ft=ruby :
Vagrant.configure("2") do |config|
  config.vm.box = "openindiana/hipster"

  # config.vm.synced_folder ".", "/vagrant_sync", type: "rsync", rsync__exclude: ".git/"

  config.vm.provision "shell", inline: <<-SHELL
    pkg install system/library/gcc-4-runtime build-essential system/library/g++-4-runtime
  SHELL

  config.vm.provision "shell", provileged: false, inline: <<-SHELL
    chmod 664 /export/home/vagrant/.bashrc
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- --profile complete -y
  SHELL
end
