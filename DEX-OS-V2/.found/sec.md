Domain,Role,Data Flow,Link Type,Risk Level,Mitigation
jeetway.vip,Frontend / CDN,User → jeetway.vip → API calls to jeetway.club,HTTPS + CORS,Medium,"Enforce strict ACAO; CSP; HSTS"
jeetway.club,Main Backend Server,Receives API from .vip; serves DB + images,Internal + Public API,High,"HA cluster; WAF; Rate limiting; JWT validation"
api.jeetway.vip,API Proxy / Mirror,Proxies requests to jeetway.club (possible CNAME),CNAME → jeetway.club,Medium,"Remove if unused; otherwise secure with mTLS"
assets.jeetway.vip,Image CDN Frontend,Fetches /assets/* via API from .club,API + CDN Cache,Medium,"Signed URLs; Cache-Control: private; no open endpoints"
jeetway.club (Image API),Image Microservice,Serves /api/getImage or /assets/*,REST API,Medium,"Auth required; short-lived tokens; IP allowlist for CDN"
Database (behind .club),Data Storage,Accessed only by .club backend,Internal VPC,Low,"VPC isolation; encrypted at rest; audit logs"
Redis/Cache (behind .club),Session / Rate Limit,Internal only,Internal,Low,"Network ACLs; no public access"