output "jdp_controller_ip" {
  value = module.JDP.controller_ip
}

output "jdp_controller_ssh" {
  value = module.JDP.controller_ssh
}

output "jdp_upgrade_worker_ip" {
  value = module.JDP.upgrade_worker_ip
}

output "jdp_upgrade_worker_ssh" {
  value = module.JDP.upgrade_worker_ssh
}

# output "tds_controller_ip" {
#   value = module.TDS.controller_ip
# }
#
# output "tds_controller_ssh" {
#   value = module.TDS.controller_ssh
# }

output "tds_upgrade_worker_ip" {
  value = module.TDS.upgrade_worker_ip
}

output "tds_upgrade_worker_ssh" {
  value = module.TDS.upgrade_worker_ssh
}