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

provider "kubernetes" {
  host                   = azurerm_kubernetes_cluster.aks.kube_config.0.host
  client_certificate     = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.client_certificate)
  client_key             = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.client_key)
  cluster_ca_certificate = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.cluster_ca_certificate)
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
    orchestrator_version = "1.30.9"
    vnet_subnet_id = azurerm_subnet.subnet.id
  }

  identity {
    type = "SystemAssigned"  # Required for extra node pools
  }

  network_profile {
    network_plugin    = "kubenet"
    network_policy    = "calico"
    service_cidr     = "172.16.0.0/16"
    dns_service_ip   = "172.16.0.10"
    pod_cidr        = "10.244.0.0/16"
  }
}

# Additional Node Pool (1 Node, 2 Pods)
resource "azurerm_kubernetes_cluster_node_pool" "fixed_pool" {
  name                  = "controller"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
  vm_size               = "Standard_B2s"
  node_count            = 1
  mode                  = "User"
}

# Autoscaling Node Pool (1-5 Nodes)
resource "azurerm_kubernetes_cluster_node_pool" "autoscaling_1" {
  name                  = "pushpool"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
  vm_size               = "Standard_B2s_v2"
  auto_scaling_enabled   = true
  node_count            = 0
  min_count             = 0
  max_count             = 5
  mode                  = "User"
}

provider "helm" {
  kubernetes {
    host                   = azurerm_kubernetes_cluster.aks.kube_config.0.host
    client_certificate     = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.client_certificate)
    client_key             = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.client_key)
    cluster_ca_certificate = base64decode(azurerm_kubernetes_cluster.aks.kube_config.0.cluster_ca_certificate)
  }
}

resource "helm_release" "keda" {
  name             = "keda"
  repository       = "https://kedacore.github.io/charts"
  chart            = "keda"
  namespace        = "keda"
  create_namespace = true
  version          = "2.12.0"

  depends_on = [azurerm_kubernetes_cluster.aks]
}
