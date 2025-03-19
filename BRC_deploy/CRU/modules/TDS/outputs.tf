# output "controller_ip" {
#   value = azurerm_public_ip.controller_ip.ip_address
# }
#
# output "controller_ssh" {
#   value = "ssh bradmin@${azurerm_public_ip.controller_ip.ip_address} -i ~/.ssh/brc"
# }

output "upgrade_worker_ip" {
  value = azurerm_network_interface.upgrade_worker_nic.private_ip_address
}

output "upgrade_worker_ssh" {
  value = "chmod 600 /home/bradmin/.ssh/brc && ssh bradmin@${azurerm_network_interface.upgrade_worker_nic.private_ip_address} -i /home/bradmin/.ssh/brc"
}

output "resource_group_name" {
  value = azurerm_resource_group.rg.name
}

output "vnet_name" {
  value = azurerm_virtual_network.vnet.name
}

output "vnet_id" {
  value = azurerm_virtual_network.vnet.id
}