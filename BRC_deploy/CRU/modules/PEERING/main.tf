terraform {
  required_providers {
    azurerm = {
      source                = "hashicorp/azurerm"
      version               = "4.23.0"
      configuration_aliases = [azurerm.jdp, azurerm.tds]
    }
  }
}

resource "azurerm_virtual_network_peering" "jdp_to_tds" {
  provider                  = azurerm.jdp
  name                      = "jdp-to-tds-peer"
  resource_group_name       = var.jdp_rg_name
  virtual_network_name      = var.jdp_vnet_name
  remote_virtual_network_id = var.tds_vnet_id
  allow_virtual_network_access = true
  allow_forwarded_traffic   = true
  allow_gateway_transit     = true
  use_remote_gateways       = true
}

resource "azurerm_virtual_network_peering" "tds_to_jdp" {
  provider                  = azurerm.tds
  name                      = "tds-to-jdp-peer"
  resource_group_name       = var.tds_rg_name
  virtual_network_name      = var.tds_vnet_name
  remote_virtual_network_id = var.jdp_vnet_id
  allow_virtual_network_access = true
  allow_forwarded_traffic   = true
  allow_gateway_transit     = true
  use_remote_gateways       = true
}