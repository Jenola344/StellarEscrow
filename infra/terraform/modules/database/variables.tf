variable "name_prefix"          { type = string }
variable "vpc_id"               { type = string }
variable "private_subnet_ids"   { type = list(string) }
variable "api_security_group_id"{ type = string }
variable "instance_class"       { type = string }
variable "allocated_storage_gb" { type = number }
variable "db_name"              { type = string }
variable "db_username"          { type = string; sensitive = true }
variable "db_password"          { type = string; sensitive = true }
variable "multi_az"             { type = bool;   default = false }
variable "deletion_protection"  { type = bool;   default = false }
