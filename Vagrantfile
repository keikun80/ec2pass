# -*- mode: ruby -*-
# vi: set ft=ruby :
# Vagrantfile for InhausLab ec2pass - Multi-Distribution Linux & Windows Environment

Vagrant.require_version ">= 2.2.0"

Vagrant.configure("2") do |config|
  # ============================================================================
  # Helper to configure Linux virtual machines
  # ============================================================================
  def self.configure_linux_node(config, name, default_box, hostname, is_primary = false)
    config.vm.define name, primary: is_primary, autostart: is_primary do |node|
      node.vm.box = ENV["VAGRANT_#{name.upcase}_BOX"] || default_box
      node.vm.hostname = hostname
      node.vm.synced_folder ".", "/vagrant", disabled: true

      # VirtualBox Provider Configuration
      node.vm.provider "virtualbox" do |vb|
        vb.name = "ec2pass-#{name}-dev"
        vb.cpus = (ENV["VM_CPUS"] || 2).to_i
        vb.memory = (ENV["VM_MEMORY"] || 4096).to_i

        # Enable Nested Hardware Virtualization (/dev/kvm support)
        vb.customize ["modifyvm", :id, "--nested-hw-virt", "on"]
        vb.customize ["modifyvm", :id, "--ioapic", "on"]
        vb.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
      end

      # Hyper-V Provider Configuration (Windows Host Native)
      node.vm.provider "hyperv" do |hv|
        hv.vmname = "ec2pass-#{name}-dev"
        hv.cpus = (ENV["VM_CPUS"] || 2).to_i
        hv.memory = (ENV["VM_MEMORY"] || 4096).to_i
        hv.maxmemory = (ENV["VM_MAX_MEMORY"] || 8192).to_i
        hv.enable_virtualization_extensions = true
      end

      node.vm.provision "shell",
        path: "packaging/vagrant/provision.sh",
        privileged: true
    end
  end

  # ============================================================================
  # Machine 1: Amazon Linux 2023 (Default Primary Linux VM)
  # ============================================================================
  configure_linux_node(config, "al2023", "bento/amazonlinux-2023", "ec2pass-al2023", true)

  # ============================================================================
  # Machine 2: Rocky Linux 9 (Latest Rocky Linux EL9 VM)
  # ============================================================================
  configure_linux_node(config, "rocky9", "bento/rockylinux-9", "ec2pass-rocky9", false)

  # ============================================================================
  # Machine 3: Debian 12 Bookworm (Latest Debian Stable VM)
  # ============================================================================
  configure_linux_node(config, "debian12", "bento/debian-12", "ec2pass-debian12", false)

  # ============================================================================
  # Machine 4: Ubuntu 24.04 LTS Noble Numbat (Latest Ubuntu LTS VM)
  # ============================================================================
  configure_linux_node(config, "ubuntu24", "bento/ubuntu-24.04", "ec2pass-ubuntu24", false)

  # ============================================================================
  # Machine 5: Windows 11 Enterprise / Pro (Opt-in Windows VM: `vagrant up win11`)
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
