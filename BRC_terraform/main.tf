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

# AKS Cluster
resource "azurerm_kubernetes_cluster" "aks" {
  name                = "brc-cluster"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  dns_prefix          = "brcaks"

  default_node_pool {
    name                = "system"
    node_count          = 1
    vm_size             = "Standard_B2ms"
    orchestrator_version = "1.28"
  }

  identity {
    type = "SystemAssigned"  # Required for extra node pools
  }
}

# Additional Node Pool (1 Node, 1 Pod, 2 Containers)
resource "azurerm_kubernetes_cluster_node_pool" "fixed_pool" {
  name                  = "controller"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
  vm_size               = "Standard_B2s"
  node_count            = 1
  mode                  = "User"
}

# # Autoscaling Node Pool (1-5 Nodes)
# resource "azurerm_kubernetes_cluster_node_pool" "autoscaling_1" {
#   name                  = "pushpool"
#   kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
#   vm_size               = "Standard_B2s_v2"
#   auto_scaling_enabled   = true
#   min_count             = 1
#   max_count             = 5
#   mode                  = "User"
# }
#
# # Autoscaling Node Pool (1-3 Nodes)
# resource "azurerm_kubernetes_cluster_node_pool" "autoscaling_2" {
#   name                  = "upgradepool"
#   kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
#   vm_size               = "Standard_B4s_v2"
#   auto_scaling_enabled   = true
#   min_count             = 1
#   max_count             = 3
#   mode                  = "User"
# }
