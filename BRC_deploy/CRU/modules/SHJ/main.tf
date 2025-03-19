terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "4.23.0"
    }
  }
}

resource "azurerm_resource_group" "rg" {
  name     = var.resource_group_name
  location = var.location
}

resource "azurerm_virtual_network" "vnet" {
  name                = var.vnet_name
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  address_space       = var.vnet_address_space
}

resource "azurerm_subnet" "subnet" {
  name                 = var.subnet_name
  resource_group_name  = azurerm_resource_group.rg.name
  virtual_network_name = azurerm_virtual_network.vnet.name
  address_prefixes     = [var.subnet_prefix]
}

resource "azurerm_public_ip" "master_push_worker_ip" {
  name                = "master-push-worker-ip"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Static"
}

resource "azurerm_network_interface" "master_push_worker_nic" {
  name                = "master-push-worker-nic"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  ip_configuration {
    name                          = "internal"
    subnet_id                     = azurerm_subnet.subnet.id
    private_ip_address_allocation = "Dynamic"
    public_ip_address_id          = azurerm_public_ip.master_push_worker_ip.id
  }
}

resource "azurerm_network_security_group" "master_push_worker_nsg" {
  name                = "master-push-worker-nsg"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  security_rule {
    name                       = "allow_ssh"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "22"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }
}

resource "azurerm_network_interface_security_group_association" "master_push_worker_nsg_assoc" {
  network_interface_id      = azurerm_network_interface.master_push_worker_nic.id
  network_security_group_id = azurerm_network_security_group.master_push_worker_nsg.id
}

resource "azurerm_linux_virtual_machine" "master_push_worker" {
  name                = "master-${var.worker_vm_name}"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = var.worker_vm_size
  admin_username      = var.admin_username
  network_interface_ids = [azurerm_network_interface.master_push_worker_nic.id]

  admin_ssh_key {
    username   = var.admin_username
    public_key = file(var.ssh_public_key_path)
  }

  os_disk {
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = var.vm_image.publisher
    offer     = var.vm_image.offer
    sku       = var.vm_image.sku
    version   = var.vm_image.version
  }

  provisioner "local-exec" {
    command = "cp ${var.worker_env_path} ${var.worker_env_modified_path} && echo 'RABBITMQ_URL=amqp://${var.rabbitmq_user}:${var.rabbitmq_password}@${var.controller_public_ip}:5672\\nQUEUE_NAME=${var.queue_name}\\nWORKER_NAME=${var.worker-1-name}' | cat - ${var.worker_env_modified_path} > temp && mv temp ${var.worker_env_modified_path}"
  }

  provisioner "file" {
    source      = var.worker_env_modified_path
    destination = "/home/${var.admin_username}/push-worker.env"
  }

  provisioner "file" {
    source      = var.ssh_private_key_path
    destination = "/home/${var.admin_username}/.ssh/brc"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt update && sudo apt install -y docker.io",
      "sudo systemctl start docker",
      "sudo usermod -aG docker ${var.admin_username}",
      "sudo chmod 666 /var/run/docker.sock",
      "sleep 30",
      "sudo docker run --env-file /home/${var.admin_username}/push-worker.env -d --name push-worker -v /var/run/docker.sock:/var/run/docker.sock ${var.worker_image}"
    ]
  }

  connection {
    type        = "ssh"
    user        = var.admin_username
    private_key = file(var.ssh_private_key_path)
    host        = azurerm_public_ip.master_push_worker_ip.ip_address
  }
}

resource "azurerm_network_interface" "upgrade_worker_nic" {
  name                = "upgrade-worker-nic"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  ip_configuration {
    name                          = "internal"
    subnet_id                     = azurerm_subnet.subnet.id
    private_ip_address_allocation = "Dynamic"
  }
}

resource "azurerm_linux_virtual_machine" "upgrade_worker" {
  name                = "upgrade-worker"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = var.upgrade-vm-size
  admin_username      = var.admin_username
  network_interface_ids = [azurerm_network_interface.upgrade_worker_nic.id]

  admin_ssh_key {
    username   = var.admin_username
    public_key = file(var.ssh_public_key_path)
  }

  os_disk {
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = var.vm_image.publisher
    offer     = var.vm_image.offer
    sku       = var.vm_image.sku
    version   = var.vm_image.version
  }

  provisioner "local-exec" {
    command = "cp ${var.worker_env_path} ${var.worker_env_modified_path} && echo 'RABBITMQ_URL=amqp://${var.rabbitmq_user}:${var.rabbitmq_password}@${var.controller_public_ip}:5672\\nQUEUE_NAME=${var.upgrade_queue_name}\\nWORKER_NAME=${var.upgrade-worker-name}' | cat - ${var.worker_env_modified_path} > temp && mv temp ${var.worker_env_modified_path}"
  }

  provisioner "file" {
    source      = var.worker_env_modified_path
    destination = "/home/${var.admin_username}/upgrade-worker.env"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt update && sudo apt install -y docker.io",
      "sudo systemctl start docker",
      "sudo usermod -aG docker ${var.admin_username}",
      "sudo chmod 666 /var/run/docker.sock",
      "sleep 30",
      "sudo docker run --env-file /home/${var.admin_username}/upgrade-worker.env -d --name upgrade-worker -v /var/run/docker.sock:/var/run/docker.sock ${var.worker_image}"
    ]
  }

  connection {
    type        = "ssh"
    user        = var.admin_username
    private_key = file(var.ssh_private_key_path)
    host        = azurerm_network_interface.upgrade_worker_nic.private_ip_address
    bastion_host = azurerm_public_ip.master_push_worker_ip.ip_address
    bastion_user = var.admin_username
    bastion_private_key = file(var.ssh_private_key_path)
  }
}