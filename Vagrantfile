# -*- mode: ruby -*-
# vi: set ft=ruby :
# Vagrantfile for InhausLab ec2pass - Amazon Linux 2023 Dev & Test Environment

Vagrant.require_version ">= 2.2.0"

Vagrant.configure("2") do |config|
  # Base Box: Amazon Linux 2023 / Enterprise Linux 9
  config.vm.box = ENV["VAGRANT_BOX"] || "bento/amazonlinux-2023"
  config.vm.hostname = "ec2pass-al2023"

  # Optional: Forward daemon RPC port (50051) only if needed for host debugging
  # config.vm.network "forwarded_port", guest: 50051, host: 50051, auto_correct: true, id: "ec2passd"

  # Disable SMB synced folder to eliminate Windows password / PIN prompts
  config.vm.synced_folder ".", "/vagrant", disabled: true

  # VirtualBox Provider Configuration
  config.vm.provider "virtualbox" do |vb|
    vb.name = "ec2pass-al2023-dev"
    vb.cpus = (ENV["VM_CPUS"] || 2).to_i
    vb.memory = (ENV["VM_MEMORY"] || 4096).to_i

    # Enable Nested Hardware Virtualization (/dev/kvm support)
    vb.customize ["modifyvm", :id, "--nested-hw-virt", "on"]
    vb.customize ["modifyvm", :id, "--ioapic", "on"]
    vb.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
  end

  # Hyper-V Provider Configuration (Windows Native)
  config.vm.provider "hyperv" do |hv|
    hv.vmname = "ec2pass-al2023-dev"
    hv.cpus = (ENV["VM_CPUS"] || 2).to_i
    hv.memory = (ENV["VM_MEMORY"] || 4096).to_i
    hv.maxmemory = (ENV["VM_MAX_MEMORY"] || 8192).to_i
    hv.enable_virtualization_extensions = true
  end

  # Automated Environment Provisioning
  config.vm.provision "shell",
    path: "packaging/vagrant/provision.sh",
    privileged: true
end
