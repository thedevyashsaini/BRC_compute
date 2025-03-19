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

module "JDP" {
  source = "./modules/JDP"

  providers = {
    azurerm = azurerm.jdp
  }

  ssh_private_key_path = "${path.module}/keys/brc"
  ssh_public_key_path  = "${path.module}/keys/brc.pub"
  queue_name = "proposal"
}

module "TDS" {
  source = "./modules/TDS"

  providers = {
    azurerm = azurerm.tds
  }

  ssh_private_key_path = "${path.module}/keys/brc"
  ssh_public_key_path  = "${path.module}/keys/brc.pub"
  queue_name = "proposal"
}

module "PEERING" {
  source = "./modules/PEERING"

  providers = {
    azurerm.jdp = azurerm.jdp
    azurerm.tds = azurerm.tds
  }

  tds_rg_name = module.TDS.resource_group_name
  tds_vnet_id = module.TDS.vnet_id
  tds_vnet_name = module.TDS.vnet_name

  jdp_rg_name   = module.JDP.resource_group_name
  jdp_vnet_id   = module.JDP.vnet_id
  jdp_vnet_name = module.JDP.vnet_name
}