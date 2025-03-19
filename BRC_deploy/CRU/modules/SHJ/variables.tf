variable "resource_group_name" {
  description = "Name of the resource group"
  type        = string
  default     = "brc-resource-grp"
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "Central India"
}

variable "vnet_name" {
  description = "Name of the virtual network"
  type        = string
  default     = "brc-network"
}

variable "vnet_address_space" {
  description = "Address space for the virtual network"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

variable "subnet_name" {
  description = "Name of the subnet"
  type        = string
  default     = "brc-subnet"
}

variable "subnet_prefix" {
  description = "Address prefix for the subnet"
  type        = string
  default     = "10.0.1.0/24"
}

variable "worker_vm_name" {
  description = "Name of the worker VM"
  type        = string
  default     = "push-worker"
}

variable "worker_vm_size" {
  description = "Size of the worker VM"
  type        = string
  default     = "Standard_B2s_v2"
}

variable "admin_username" {
  description = "Admin username for VMs"
  type        = string
  default     = "bradmin"
}

variable "ssh_public_key_path" {
  description = "Path to the SSH public key file"
  type        = string
}

variable "ssh_private_key_path" {
  description = "Path to the SSH private key file"
  type        = string
}

variable "worker_env_path" {
  description = "Path to the worker environment file"
  type        = string
  default     = "vars/worker.env"
}

variable "worker_env_modified_path" {
  description = "Path to the modified worker environment file"
  type        = string
  default     = "vars/worker-modified.env"
}

variable "rabbitmq_user" {
  description = "Username for RabbitMQ"
  type        = string
  default     = "admin"
}

variable "rabbitmq_password" {
  description = "Password for RabbitMQ"
  type        = string
  default     = "1121"
  sensitive   = true
}

variable "queue_name" {
  description = "Name of the RabbitMQ queue"
  type        = string
  default     = "proporal"
}

variable "upgrade_queue_name" {
    description = "Name of the RabbitMQ queue for upgrade worker"
    type        = string
    default     = "divorce"
}

variable "worker_image" {
  description = "Docker image for worker"
  type        = string
  default     = "steakfisher1/brc-worker"
}

variable "vm_image" {
  description = "VM image configuration"
  type        = object({
    publisher = string
    offer     = string
    sku       = string
    version   = string
  })
  default     = {
    publisher = "Canonical"
    offer     = "UbuntuServer"
    sku       = "18.04-LTS"
    version   = "latest"
  }
}

variable "controller_public_ip" {
  description = "Private IP of the JDP controller VM (for RabbitMQ connection)"
  type        = string
}

variable "worker-1-name" {
  description = "Name of the first worker VM"
  type        = string
  default     = "push-worker-1"
}

variable "upgrade-worker-name" {
    description = "Name of the upgrade worker VM"
    type        = string
    default     = "upgrade-worker"
}

variable "upgrade-vm-size" {
    description = "Size of the upgrade VM"
    type        = string
    default     = "Standard_B4s_v2"
}