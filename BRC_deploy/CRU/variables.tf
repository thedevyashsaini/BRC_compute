variable "tds_subscription_id" {
  description = "Azure subscription ID for TDS provider"
  type        = string
}

variable "tds_tenant_id" {
  description = "Azure tenant ID for TDS provider"
  type        = string
}

variable "tds_client_id" {
  description = "Azure client ID for TDS provider"
  type        = string
}

variable "tds_client_secret" {
  description = "Azure client secret for TDS provider"
  type        = string
  sensitive   = true
}

variable "shj_subscription_id" {
  description = "Azure subscription ID for SHJ provider"
  type        = string
}

variable "shj_tenant_id" {
  description = "Azure tenant ID for SHJ provider"
  type        = string
}

variable "shj_client_id" {
  description = "Azure client ID for SHJ provider"
  type        = string
}

variable "shj_client_secret" {
  description = "Azure client secret for SHJ provider"
  type        = string
  sensitive   = true
}

variable "cloudflare_zone_id" {
  description = "Cloudflare zone ID"
  type        = string
}

variable "cloudflare_record_id" {
  description = "Cloudflare record ID"
  type        = string
}

variable "cloudflare_token" {
  description = "Cloudflare API token"
  type        = string
  sensitive   = true
}
