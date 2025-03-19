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

output "tds_master_push_worker_public_ip" {
  value = module.TDS.master_push_worker_public_ip
}

output "tds_master_push_worker_ssh" {
  value = module.TDS.master_push_worker_ssh
}

output "tds_push_worker_1_private_ip" {
  value = module.TDS.push_worker_1_private_ip
}

output "tds_push_worker_1_ssh" {
  value = module.TDS.push_worker_1_ssh
}

output "tds_push_worker_2_private_ip" {
  value = module.TDS.push_worker_2_private_ip
}

output "tds_push_worker_2_ssh" {
  value = module.TDS.push_worker_2_ssh
}

output "shj_master_push_worker_public_ip" {
  value = module.SHJ.master_push_worker_public_ip
}

output "shj_master_push_worker_ssh" {
  value = module.SHJ.master_push_worker_ssh
}

output "shj_upgrade_worker_private_ip" {
  value = module.SHJ.upgrade_worker_private_ip
}

output "shj_upgrade_worker_ssh" {
  value = module.SHJ.upgrade_worker_ssh
}