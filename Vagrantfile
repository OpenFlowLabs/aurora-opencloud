# -*- mode: ruby -*-
# vi: set ft=ruby :
Vagrant.configure("2") do |config|
  config.vm.box = "openindiana/hipster"

	# Autoconfigure resources for development VM. The snippet is taken from
    # https://stefanwrobel.com/how-to-make-vagrant-performance-not-suck.
    # We allocate 1/4 of available system memory and CPU core count of the host
    # to the VM, so performance does not suck.
    host = RbConfig::CONFIG['host_os']

    # Get memory size and CPU cores amount
    if host =~ /solaris/
        mem = `/usr/sbin/prtconf|grep Memory|cut -f3 -d' '`.to_i * 1024 * 1024
        cpus = `/usr/sbin/psrinfo|wc -l`.to_i
    elsif host =~ /darwin/
        # sysctl returns Bytes
        mem = `sysctl -n hw.memsize`.to_i
        cpus = `sysctl -n hw.ncpu`.to_i
    elsif host =~ /linux/
        # meminfo shows size in kB; convert to Bytes
        mem = `awk '/MemTotal/ {print $2}' /proc/meminfo`.to_i * 1024
        cpus = `getconf _NPROCESSORS_ONLN`.to_i
    elsif host =~ /mswin|mingw|cygwin/
        # Windows code via https://github.com/rdsubhas/vagrant-faster
        mem = `wmic computersystem Get TotalPhysicalMemory`.split[1].to_i
        cpus = `echo %NUMBER_OF_PROCESSORS%`.to_i
    else
        puts "Unsupported operating system"
        exit
    end

    # Give VM 1/4 system memory as well as CPU core count
    mem /= 1024 ** 2 * 4
    cpus /= 4  

  config.vm.provider "virtualbox" do |v|
        v.customize ["modifyvm", :id, "--memory", mem]
        v.customize ["modifyvm", :id, "--cpus", cpus]
        v.customize ['modifyvm', :id, '--nested-hw-virt', 'on']
        v.customize ["storagectl", :id, "--name", "SATA Controller", "--hostiocache", "on"]
        # Enable following line, if oi-userland directory is on non-rotational
        # drive (e.g. SSD). (This could be automated, but with all those storage
        # technologies (LVM, partitions, ...) on all three operationg systems,
        # it's actually error prone to detect it automatically.) macOS has it
        # enabled by default as recent Macs have SSD anyway.
        if host =~ /darwin/
            v.customize ["storageattach", :id, "--storagectl", "SATA Controller", "--port", 0, "--nonrotational", "on"]
        else
            #v.customize ["storageattach", :id, "--storagectl", "SATA Controller", "--port", 0, "--nonrotational", "on"]
        end
        # Should we ever support `--discard` option, we need to switch to VDI
        # virtual disk format first.
        #v.customize ["storageattach", :id, "--storagectl", "SATA Controller", "--port", 0, "--discard", "on"]

    end

    config.vm.provider :libvirt do |libvirt|
        libvirt.memory = mem
        libvirt.cpus = cpus
    end


  config.vm.provision "shell", inline: <<-SHELL
    set -ex
    pkg install system/library/gcc-10-runtime build-essential system/library/g++-10-runtime system/library/gcc-10-compat-links jq
    mkdir /ws
    chown vagrant:vagrant /ws
    zfs create -o mountpoint=/zones rpool/zones
  SHELL

  config.vm.provision "shell", privileged: false, inline: <<-SHELL
    set -ex
    chmod 664 $HOME/.bashrc
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- --profile complete -y
    mkdir -p /export/home/vagrant/.cargo/
    cat<<EOF > /export/home/vagrant/.cargo/config.toml
[build]
target-dir = "/export/home/vagrant/.cargo/target"
EOF
  SHELL

end
