# -*- mode: ruby -*-
# vi: set ft=ruby :
# Vagrantfile for InhausLab ec2pass - Multi-Platform (Amazon Linux 2023 & Windows 11) Environment

Vagrant.require_version ">= 2.2.0"

Vagrant.configure("2") do |config|
  # ============================================================================
  # Machine 1: Amazon Linux 2023 (Default Primary Linux VM)
  # ============================================================================
  config.vm.define "al2023", primary: true do |al|
    al.vm.box = ENV["VAGRANT_BOX"] || "bento/amazonlinux-2023"
    al.vm.hostname = "ec2pass-al2023"
    al.vm.synced_folder ".", "/vagrant", disabled: true

    # VirtualBox Provider Configuration
    al.vm.provider "virtualbox" do |vb|
      vb.name = "ec2pass-al2023-dev"
      vb.cpus = (ENV["VM_CPUS"] || 2).to_i
      vb.memory = (ENV["VM_MEMORY"] || 4096).to_i

      # Enable Nested Hardware Virtualization (/dev/kvm support)
      vb.customize ["modifyvm", :id, "--nested-hw-virt", "on"]
      vb.customize ["modifyvm", :id, "--ioapic", "on"]
      vb.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
    end

    # Hyper-V Provider Configuration (Windows Host Native)
    al.vm.provider "hyperv" do |hv|
      hv.vmname = "ec2pass-al2023-dev"
      hv.cpus = (ENV["VM_CPUS"] || 2).to_i
      hv.memory = (ENV["VM_MEMORY"] || 4096).to_i
      hv.maxmemory = (ENV["VM_MAX_MEMORY"] || 8192).to_i
      hv.enable_virtualization_extensions = true
    end

    al.vm.provision "shell",
      path: "packaging/vagrant/provision.sh",
      privileged: true
  end

  # ============================================================================
  # Machine 2: Windows 11 Enterprise / Pro (Opt-in Windows VM: `vagrant up win11`)
  # ============================================================================
  config.vm.define "win11", autostart: false do |win|
    win.vm.box = ENV["VAGRANT_WIN11_BOX"] || "gusztavvargadr/windows-11"
    win.vm.hostname = "ec2pass-win11"
    win.vm.communicator = "winrm"
    win.vm.boot_timeout = (ENV["VM_WIN11_BOOT_TIMEOUT"] || 900).to_i
    win.vm.synced_folder ".", "/vagrant", disabled: true

    # VirtualBox Configuration with Windows 11 TPM 2.0 & EFI Emulation
    win.vm.provider "virtualbox" do |vb|
      vb.name = "ec2pass-win11-dev"
      vb.cpus = (ENV["VM_WIN11_CPUS"] || 4).to_i
      vb.memory = (ENV["VM_WIN11_MEMORY"] || 6144).to_i

      vb.customize ["modifyvm", :id, "--firmware", "efi"]
      vb.customize ["modifyvm", :id, "--tpm-type", "2.0"]
      vb.customize ["modifyvm", :id, "--nested-hw-virt", "on"]
      vb.customize ["modifyvm", :id, "--vram", "128"]
      vb.customize ["modifyvm", :id, "--graphicscontroller", "vboxvmsvga"]
      vb.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
    end

    win.vm.provision "shell",
      path: "packaging/vagrant/provision_win11.ps1"
  end
end
