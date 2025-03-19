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
  queue_name = "divorce"
  worker_name = "Tom"
}

module "TDS" {
  source = "./modules/TDS"

  providers = {
    azurerm = azurerm.tds
  }

  ssh_private_key_path = "${path.module}/keys/brc"
  ssh_public_key_path  = "${path.module}/keys/brc.pub"
  queue_name = "proposal"
  controller_public_ip = module.JDP.controller_ip
  worker-1-name = "Akkad"
  worker-2-name = "Bakkad"
  worker-3-name = "Bambey"
}

module "SHJ" {
  source = "./modules/SHJ"

  providers = {
    azurerm = azurerm.shj
  }

  ssh_private_key_path = "${path.module}/keys/brc"
  ssh_public_key_path  = "${path.module}/keys/brc.pub"
  queue_name = "proposal"
  upgrade_queue_name = "divorce"
  controller_public_ip = module.JDP.controller_ip
  worker-1-name = "Bow"
  upgrade-worker-name = "Jerry"
}