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

resource "azurerm_public_ip" "controller_ip" {
  name                = "${var.controller_vm_name}-ip"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Static"
}

resource "azurerm_network_interface" "controller_nic" {
  name                =  "${var.controller_vm_name}-nic"
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
  name                = "${var.controller_vm_name}-nsg"
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
Shoul    destination_port_ranges    = ["80", "443"]
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
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }
}

resource "azurerm_network_interface_security_group_association" "controller_nsg_assoc" {
  network_interface_id      = azurerm_network_interface.controller_nic.id
  network_security_group_id = azurerm_network_security_group.controller_nsg.id
}

resource "azurerm_linux_virtual_machine" "controller" {
  name                = var.controller_vm_name
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = var.controller_vm_size
  admin_username      = var.admin_username
  network_interface_ids = [azurerm_network_interface.controller_nic.id]

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

  provisioner "file" {
    source      = var.controller_env_path
    destination = "/home/${var.admin_username}/controller.env"
  }

  provisioner "file" {
    source      = var.ssh_private_key_path
    destination = "/home/${var.admin_username}/.ssh/brc"
  }

  provisioner "file" {
    source      = var.nginx_config_path
    destination = "/home/${var.admin_username}/api.brc.r00t3d.co"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt update && sudo apt install -y certbot",
      "sudo mv /home/${var.admin_username}/api.brc.r00t3d.co /etc/nginx/sites-available/ && sudo chmod 644 /etc/nginx/sites-available/api.brc.r00t3d.co",
      # "curl -X PUT \"https://api.cloudflare.com/client/v4/zones/${var.cloudflare_zone_id}/dns_records/${var.cloudflare_record_id}\" -H \"Authorization: Bearer ${var.cloudflare_token}\" -H \"Content-Type: application/json\" --data '{\"type\":\"A\",\"name\":\"api.brc.r00t3d.co\",\"content\":\"'\"$(curl -s https://api64.ipify.org)\"'\",\"ttl\":120}'",
      # "sleep 10",
      # "sudo certbot certonly --standalone -d api.brc.r00t3d.co --agree-tos --email dysaini2004@gmail.com --non-interactive",
      "sudo apt install -y docker.io nginx",
      "sudo curl -o /etc/letsencrypt/options-ssl-nginx.conf https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf",
      "sudo curl -o /etc/letsencrypt/ssl-dhparams.pem https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem",
      # "sudo systemctl start nginx",
      "sudo systemctl start docker",
      "sudo docker network create brc-network",
      "sudo docker run --network brc-network --env-file /home/${var.admin_username}/controller.env -p 5000:5000 -d --name controller ${var.controller_image}",
      "sudo docker run --network brc-network -d -p 5672:5672 -p 15672:15672 --name rabbitmq -e RABBITMQ_DEFAULT_USER=${var.rabbitmq_user} -e RABBITMQ_DEFAULT_PASS=${var.rabbitmq_password} -e RABBITMQ_NODENAME=rabbit rabbitmq:management",
      # "sudo ln -s /etc/nginx/sites-available/api.brc.r00t3d.co /etc/nginx/sites-enabled/",
      # "sudo systemctl reload nginx"
    ]
  }

  connection {
    type        = "ssh"
    user        = var.admin_username
    private_key = file(var.ssh_private_key_path)
    host        = azurerm_public_ip.controller_ip.ip_address
  }
}

resource "azurerm_network_interface" "upgrade_worker_nic" {
  name                = "${var.worker_vm_name}-nic"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  ip_configuration {
    name                          = "internal"
    subnet_id                     = azurerm_subnet.subnet.id
    private_ip_address_allocation = "Dynamic"
  }
}

resource "azurerm_linux_virtual_machine" "upgrade_worker" {
  name                = var.worker_vm_name
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = var.worker_vm_size
  admin_username      = var.admin_username
  network_interface_ids = [azurerm_network_interface.upgrade_worker_nic.id]
  depends_on = [azurerm_linux_virtual_machine.controller]

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
    command = "cp ${var.worker_env_path} ${var.worker_env_modified_path} && echo 'RABBITMQ_URL=amqp://${var.rabbitmq_user}:${var.rabbitmq_password}@${azurerm_network_interface.controller_nic.private_ip_address}:5672\nQUEUE_NAME=${var.queue_name}\nWORKER_NAME=${var.worker_name}' | cat - ${var.worker_env_modified_path} > temp && mv temp ${var.worker_env_modified_path}"
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
    bastion_host = azurerm_public_ip.controller_ip.ip_address
    bastion_user = var.admin_username
    bastion_private_key = file(var.ssh_private_key_path)
  }
}