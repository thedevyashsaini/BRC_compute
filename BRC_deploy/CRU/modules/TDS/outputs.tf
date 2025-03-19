output "master_push_worker_public_ip" {
  value = azurerm_public_ip.master_push_worker_ip.ip_address
}

output "master_push_worker_ssh" {
  value = "ssh bradmin@${azurerm_public_ip.master_push_worker_ip.ip_address} -i ~/.ssh/brc"
}

output "push_worker_1_private_ip" {
  value = azurerm_network_interface.push_worker_1_nic.private_ip_address
}

output "push_worker_1_ssh" {
  value = "chmod 600 /home/bradmin/.ssh/brc && ssh bradmin@${azurerm_network_interface.push_worker_1_nic.private_ip_address} -i /home/bradmin/.ssh/brc"
}

output "push_worker_2_private_ip" {
  value = azurerm_network_interface.push_worker_2_nic.private_ip_address
}

output "push_worker_2_ssh" {
  value = "chmod 600 /home/bradmin/.ssh/brc && ssh bradmin@${azurerm_network_interface.push_worker_2_nic.private_ip_address} -i /home/bradmin/.ssh/brc"
}