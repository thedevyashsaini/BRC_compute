terraform { 
  cloud { 
    
    organization = "Painsicle" 

    workspaces { 
      name = "BRC-Compute" 
    } 
  } 
}

provider "azurerm" {
  features {}
}

provider "helm" {
  kubernetes {
    host                   = azurerm_kubernetes_cluster.brc_aks.kube_config.0.host
    client_certificate     = base64decode(azurerm_kubernetes_cluster.brc_aks.kube_config.0.client_certificate)
    client_key             = base64decode(azurerm_kubernetes_cluster.brc_aks.kube_config.0.client_key)
    cluster_ca_certificate = base64decode(azurerm_kubernetes_cluster.brc_aks.kube_config.0.cluster_ca_certificate)
  }
}

resource "azurerm_resource_group" "brc_rg" {
  name     = "brc-resource-group"
  location = "Central India"
}

resource "azurerm_kubernetes_cluster" "brc_aks" {
  name                = "brc-cluster"
  location            = azurerm_resource_group.brc_rg.location
  resource_group_name = azurerm_resource_group.brc_rg.name
  dns_prefix          = "brcaks"

  default_node_pool {
    name                = "default"
    node_count          = 1
    vm_size             = "Standard_DS2_v2"
    min_count           = 1
    max_count           = 3
    vnet_subnet_id      = azurerm_subnet.aks_subnet.id
  }

  network_profile {
    network_plugin = "azure"
    network_policy = "azure"
  }

  identity {
    type = "SystemAssigned"
  }

  tags = {
    environment = "BRC"
  }
}

# Node Pool 1 - Controller + RabbitMQ
resource "azurerm_kubernetes_cluster_node_pool" "nodepool1" {
  name                  = "controller"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.brc_aks.id
  vm_size               = "Standard_DS2_v2"
  node_count            = 2
}

# Node Pool 2 - Worker Type Push
resource "azurerm_kubernetes_cluster_node_pool" "nodepool2" {
  name                  = "worker-push"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.brc_aks.id
  vm_size               = "Standard_B2s"
  min_count             = 1
  max_count             = 2
}

# Node Pool 3 - Worker Type Upgrade
resource "azurerm_kubernetes_cluster_node_pool" "nodepool3" {
  name                  = "worker-upgrade"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.brc_aks.id
  vm_size               = "Standard_D4s_v3"
  min_count             = 1
  max_count             = 2
}

resource "azurerm_virtual_network" "brc_vnet" {
  name                = "brc-vnet"
  location            = azurerm_resource_group.brc_rg.location
  resource_group_name = azurerm_resource_group.brc_rg.name
  address_space       = ["10.0.0.0/16"]
}

resource "azurerm_subnet" "aks_subnet" {
  name                 = "aks-subnet"
  resource_group_name  = azurerm_resource_group.brc_rg.name
  virtual_network_name = azurerm_virtual_network.brc_vnet.name
  address_prefixes     = ["10.0.1.0/24"]
}

resource "helm_release" "nginx_ingress" {
  name       = "nginx-ingress"
  repository = "https://kubernetes.github.io/ingress-nginx"
  chart      = "ingress-nginx"
  
  set {
    name  = "controller.service.annotations.service\\.beta\\.kubernetes\\.io/azure-dns-label-name"
    value = "brc-controller"  # This will create a DNS name: brc-controller.<region>.cloudapp.azure.com
  }
  
  depends_on = [
    azurerm_kubernetes_cluster.brc_aks
  ]
}

resource "helm_release" "vault" {
  name       = "vault"
  repository = "https://helm.releases.hashicorp.com"
  chart      = "vault"
}