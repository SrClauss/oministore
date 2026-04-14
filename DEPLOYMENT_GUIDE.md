# 🚀 Guia de Deploy - OmniStore Headless API

> **Público-alvo**: MEIs e desenvolvedores implantando em VPS  
> **Tempo estimado**: 30-45 minutos  
> **Custo**: A partir de R$ 25/mês (VPS básico)

---

## 📋 Pré-requisitos

### O que você vai precisar

- ✅ **VPS com Linux** (Ubuntu 22.04 ou similar)
- ✅ **Mínimo 1GB RAM** (recomendado 2GB)
- ✅ **10GB de espaço** em disco
- ✅ **Domínio** (opcional, mas recomendado)
- ✅ **Conhecimentos básicos** de terminal Linux

### Provedores Recomendados

| Provedor | Plano | RAM | CPU | Disco | Preço/mês |
|----------|-------|-----|-----|-------|-----------|
| **Hetzner** | CX11 | 2GB | 1 vCPU | 20GB | ~€4 (R$23) |
| **DigitalOcean** | Basic | 1GB | 1 vCPU | 25GB | $6 (R$30) |
| **Contabo** | VPS S | 4GB | 2 vCPU | 50GB | €5 (R$29) |
| **Vultr** | Regular | 1GB | 1 vCPU | 25GB | $6 (R$30) |

💡 **Recomendação**: Hetzner (melhor custo-benefício) ou DigitalOcean (mais fácil para iniciantes)

---

## 📦 Opção 1: Deploy Rápido (Docker Compose)

### Passo 1: Conectar ao VPS

```bash
# Substitua pelo IP do seu servidor
ssh root@seu-servidor.com
```

### Passo 2: Instalar Docker

```bash
# Atualizar pacotes
apt update && apt upgrade -y

# Instalar Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Instalar Docker Compose
apt install docker-compose -y

# Verificar instalação
docker --version
docker-compose --version
```

### Passo 3: Clonar e Configurar

```bash
# Clonar repositório
git clone https://github.com/seu-usuario/omnistore.git
cd omnistore

# Copiar arquivo de configuração
cp .env.example .env

# Editar variáveis de ambiente
nano .env
```

### Passo 4: Configurar `.env`

```bash
# Configurações essenciais
APP_PORT=8080
APP_ENV=production

# MongoDB (não precisa mudar se usar Docker)
MONGO_URI=mongodb://admin:senha_forte_aqui@mongo:27017
MONGO_DB_NAME=omnistore

# Redis (não precisa mudar se usar Docker)
REDIS_URL=redis://redis:6379

# MinIO / S3 (para upload de imagens)
S3_ENDPOINT=http://minio:9000
S3_BUCKET=products
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin_senha_forte

# Mercado Pago (pegue em https://www.mercadopago.com.br/developers)
MERCADO_PAGO_ACCESS_TOKEN=seu_token_aqui

# Asaas (pegue em https://www.asaas.com/api)
ASAAS_API_KEY=sua_chave_aqui

# Token de Admin (crie uma senha forte)
ADMIN_TOKEN=troque_por_uma_senha_forte_aleatoria
```

💡 **Dica**: Use `openssl rand -base64 32` para gerar senhas fortes

### Passo 5: Otimizar Docker Compose para VPS

Edite o `docker-compose.yaml` para limitar memória:

```bash
nano docker-compose.yaml
```

Adicione limites de memória em cada serviço:

```yaml
services:
  app:
    # ...existente...
    deploy:
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M

  mongo:
    # ...existente...
    deploy:
      resources:
        limits:
          memory: 256M
        reservations:
          memory: 128M
    command: mongod --wiredTigerCacheSizeGB 0.25

  redis:
    # ...existente...
    deploy:
      resources:
        limits:
          memory: 128M
        reservations:
          memory: 64M

  minio:
    # ...existente...
    deploy:
      resources:
        limits:
          memory: 128M
        reservations:
          memory: 64M
```

### Passo 6: Iniciar Aplicação

```bash
# Subir todos os containers
docker-compose up -d

# Verificar logs
docker-compose logs -f app

# Verificar se está rodando
curl http://localhost:8080/health
```

Se ver `{"status":"ok"}`, está funcionando! 🎉

---

## 🌐 Opção 2: Expor com Nginx + SSL

### Passo 1: Instalar Nginx

```bash
apt install nginx certbot python3-certbot-nginx -y
```

### Passo 2: Configurar Nginx

```bash
nano /etc/nginx/sites-available/omnistore
```

Cole esta configuração:

```nginx
server {
    listen 80;
    server_name api.sualoja.com.br;

    # Aumentar limite de upload (para imagens de produtos)
    client_max_body_size 20M;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Ativar configuração:

```bash
ln -s /etc/nginx/sites-available/omnistore /etc/nginx/sites-enabled/
nginx -t  # Testar configuração
systemctl restart nginx
```

### Passo 3: Configurar SSL (HTTPS)

```bash
# Obter certificado SSL gratuito (Let's Encrypt)
certbot --nginx -d api.sualoja.com.br

# Renovação será automática!
```

Pronto! Sua API está em `https://api.sualoja.com.br` 🔒

---

## 🔧 Configuração de DNS

No seu provedor de domínio (Registro.br, GoDaddy, etc):

```
Tipo: A
Nome: api (ou @)
Valor: IP.DO.SEU.VPS
TTL: 3600
```

Aguarde 5-15 minutos para propagação.

---

## 🛡️ Hardening Básico (Segurança)

### Firewall

```bash
# Permitir apenas portas necessárias
ufw allow 22    # SSH
ufw allow 80    # HTTP
ufw allow 443   # HTTPS
ufw enable
```

### Criar usuário não-root

```bash
# Criar usuário
adduser deploy
usermod -aG sudo deploy
usermod -aG docker deploy

# Testar login
su - deploy
docker ps  # Deve funcionar sem sudo
```

### Desabilitar login root por SSH

```bash
nano /etc/ssh/sshd_config

# Mudar esta linha:
PermitRootLogin no

# Reiniciar SSH
systemctl restart sshd
```

---

## 📊 Monitoramento Simples

### Ver logs em tempo real

```bash
# Logs da aplicação
docker-compose logs -f app

# Logs do MongoDB
docker-compose logs -f mongo

# Ver uso de recursos
docker stats
```

### Criar script de health check

```bash
nano /home/deploy/check-health.sh
```

```bash
#!/bin/bash
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)

if [ $RESPONSE -ne 200 ]; then
    echo "API down! Restarting..."
    cd /home/deploy/omnistore
    docker-compose restart app
    
    # Enviar notificação (opcional)
    # curl -X POST https://api.telegram.org/botTOKEN/sendMessage \
    #   -d chat_id=SEU_ID \
    #   -d text="OmniStore API foi reiniciada"
fi
```

```bash
chmod +x /home/deploy/check-health.sh

# Adicionar ao cron (verificar a cada 5 minutos)
crontab -e
```

Adicione:
```
*/5 * * * * /home/deploy/check-health.sh
```

---

## 💾 Backup Automático

### Script de backup do MongoDB

```bash
nano /home/deploy/backup-mongo.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/home/deploy/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Criar diretório se não existir
mkdir -p $BACKUP_DIR

# Fazer backup
docker exec omnistore-mongo mongodump \
  --uri="mongodb://admin:senha@localhost:27017" \
  --out=/dump

# Copiar para host
docker cp omnistore-mongo:/dump $BACKUP_DIR/mongo_$DATE

# Compactar
cd $BACKUP_DIR
tar -czf mongo_$DATE.tar.gz mongo_$DATE
rm -rf mongo_$DATE

# Manter apenas últimos 7 dias
find $BACKUP_DIR -name "mongo_*.tar.gz" -mtime +7 -delete

echo "Backup concluído: mongo_$DATE.tar.gz"
```

```bash
chmod +x /home/deploy/backup-mongo.sh

# Executar todo dia às 3h da manhã
crontab -e
```

Adicione:
```
0 3 * * * /home/deploy/backup-mongo.sh
```

---

## 🔄 Atualizar Aplicação

```bash
cd /home/deploy/omnistore

# Baixar atualizações
git pull

# Reconstruir imagem
docker-compose build app

# Reiniciar (sem downtime em produção use blue-green)
docker-compose up -d app

# Ver se está saudável
docker-compose logs -f app
```

---

## 🆘 Troubleshooting

### API não responde

```bash
# Verificar se container está rodando
docker-compose ps

# Ver logs de erro
docker-compose logs app

# Reiniciar tudo
docker-compose restart
```

### MongoDB sem espaço em disco

```bash
# Ver uso de disco
df -h

# Limpar logs antigos
docker system prune -a

# Compactar banco MongoDB
docker exec omnistore-mongo mongo --eval "db.runCommand({compact: 'products'})"
```

### Imagens não aparecem

```bash
# Verificar MinIO
docker-compose logs minio

# Acessar console MinIO
# http://SEU-IP:9001
# Login: minioadmin / (senha do .env)

# Verificar se bucket 'products' existe
```

### Erro de memória

```bash
# Ver uso de RAM
free -h

# Ver uso por container
docker stats

# Adicionar swap (emergência)
fallocate -l 1G /swapfile
chmod 600 /swapfile
mkswap /swapfile
swapon /swapfile
echo '/swapfile none swap sw 0 0' >> /etc/fstab
```

---

## 📈 Próximos Passos

Depois do deploy básico:

1. ✅ **Testar endpoints** com Postman/Insomnia
2. ✅ **Configurar Mercado Pago/Asaas** com credenciais reais
3. ✅ **Criar frontend** (Next.js, React, Vue, etc)
4. ✅ **Configurar domínio** personalizado
5. ✅ **Monitoramento** com UptimeRobot (grátis)
6. ✅ **Backup** para S3/Dropbox/Google Drive

---

## 💰 Custos Mensais Estimados

| Item | Custo |
|------|-------|
| VPS (Hetzner 2GB) | R$ 23 |
| Domínio (.com.br) | R$ 40/ano = R$ 3/mês |
| SSL (Let's Encrypt) | Grátis |
| **Total** | **~R$ 26/mês** |

🎯 **Excelente custo-benefício para MEIs!**

---

## 📚 Recursos Adicionais

- [Docker Docs](https://docs.docker.com/)
- [Nginx Docs](https://nginx.org/en/docs/)
- [Let's Encrypt](https://letsencrypt.org/)
- [DigitalOcean Tutorials](https://www.digitalocean.com/community/tutorials)

---

**Dúvidas?** Abra uma issue no GitHub ou consulte [SIMPLE_SETUP.md](SIMPLE_SETUP.md) para guia visual passo-a-passo.

🚀 **Bom deploy!**
