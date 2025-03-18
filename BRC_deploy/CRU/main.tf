terraform {
  cloud {
    organization = "SteakFishersOrg"

    workspaces {
      name = "BRC_Workspace"
    }
  }

  required_providers {
    azurerm = {
      source = "hashicorp/azurerm"
      version = "4.23.0"
    }
  }
}

provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "rg" {
  name     = "brc-resource-grp"
  location = "Central India"
}

resource "azurerm_virtual_network" "vnet" {
  name                = "brc-network"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  address_space       = ["10.0.0.0/16"]
}

resource "azurerm_subnet" "subnet" {
  name                 = "brc-subnet"
  resource_group_name  = azurerm_resource_group.rg.name
  virtual_network_name = azurerm_virtual_network.vnet.name
  address_prefixes     = ["10.0.1.0/24"]
}

resource "azurerm_public_ip" "controller_ip" {
  name                = "controller-ip"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Static"
}

resource "azurerm_network_interface" "controller_nic" {
  name                = "controller-nic"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  ip_configuration {
    name                          = "controller-ip-config"
    subnet_id                     = azurerm_subnet.subnet.id
    private_ip_address_allocation = "Dynamic"
    public_ip_address_id          = azurerm_public_ip.controller_ip.id
  }
}

resource "azurerm_network_security_group" "controller_nsg" {
  name                = "controller-nsg"
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

  security_rule {
    name                       = "allow_5000"
    priority                   = 110
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "5000"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  security_rule {
    name                       = "allow_rabbitmq_vnet_only"
    priority                   = 120
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_ranges    = ["5672", "15672"]
    source_address_prefix      = "VirtualNetwork"
    destination_address_prefix = "*"
  }
}

resource "azurerm_network_interface_security_group_association" "controller_nsg_assoc" {
  network_interface_id      = azurerm_network_interface.controller_nic.id
  network_security_group_id = azurerm_network_security_group.controller_nsg.id
}

resource "azurerm_linux_virtual_machine" "controller" {
  name                = "controller"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = "Standard_B2s_v2"
  admin_username      = "bradmin"
  network_interface_ids = [azurerm_network_interface.controller_nic.id]

  admin_ssh_key {
    username   = "bradmin"
    public_key = file("${path.module}/keys/brc.pub")
  }

  os_disk {
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = "Canonical"
    offer     = "UbuntuServer"
    sku       = "18.04-LTS"
    version   = "latest"
  }

  provisioner "file" {
    source      = "./vars/controller.env" # Replace
    destination = "/home/bradmin/controller.env" # Replace
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt update && sudo apt install -y docker.io",
      "sudo systemctl start docker",
      "sudo docker network create brc-network",
      "sudo docker run --network brc-network --env-file /home/bradmin/controller.env -p 5000:5000 -d --name controller steakfisher1/brc-controller",
      "sudo docker run --network brc-network -d -p 5672:5672 -p 15672:15672 --name rabbitmq rabbitmq:management"
    ]
  }

  connection {
    type        = "ssh"
    user        = "bradmin"
    private_key = file("${path.module}/keys/brc") # Replace
    host        = azurerm_public_ip.controller_ip.ip_address
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
  size                = "Standard_B4s_v2"
  admin_username      = "bradmin"
  network_interface_ids = [azurerm_network_interface.upgrade_worker_nic.id]

  admin_ssh_key {
    username   = "bradmin"
    public_key = file("${path.module}/keys/brc.pub")
  }

  os_disk {
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = "Canonical"
    offer     = "UbuntuServer"
    sku       = "18.04-LTS"
    version   = "latest"
  }

  provisioner "local-exec" {
    command = "cp ./vars/upgrade-worker.env ./vars/upgrade-worker-modified.env && echo 'RABBITMQ_URL=amqp://${azurerm_network_interface.controller_nic.private_ip_address}\nQUEUE_NAME=divorce' | cat - ./vars/upgrade-worker-modified.env > temp && mv temp ./vars/upgrade-worker-modified.env"
  }

  provisioner "file" {
    source      = "./vars/upgrade-worker-modified.env"
    destination = "/home/bradmin/upgrade-worker.env"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt update && sudo apt install -y docker.io",
      "sudo systemctl start docker",
      "sudo docker run --env-file /home/bradmin/upgrade-worker.env -d --name upgrade-worker steakfisher1/brc-worker"
    ]
  }

  connection {
    type        = "ssh"
    user        = "bradmin"
    private_key = file("${path.module}/keys/brc")
    host        = azurerm_network_interface.upgrade_worker_nic.private_ip_address
    bastion_host = azurerm_public_ip.controller_ip.ip_address
    bastion_user = "bradmin"
    bastion_private_key = file("${path.module}/keys/brc")
  }
}
