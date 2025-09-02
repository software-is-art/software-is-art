resource "cloudflare_record" "wildcard" {
  zone_id = var.zone_id
  name    = "*"
  type    = "CNAME"
  value   = "example.com"
  proxied = true
}
