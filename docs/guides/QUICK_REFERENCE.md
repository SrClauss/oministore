# 🚀 Referência Rápida - OmniStore

Comandos e snippets úteis para desenvolvimento diário.

---

## 🏃 Quick Start

```bash
# Clone o repositório
git clone <repo-url>
cd omnistore

# Copie o .env de exemplo
cp .env.example .env

# Edite as variáveis de ambiente
nano .env  # ou seu editor favorito

# Inicie os serviços
docker-compose up -d

# Veja os logs
docker-compose logs -f app

# Acesse a API
curl http://localhost:8080
```

---

## 🔨 Comandos de Desenvolvimento

### Rust / Cargo

```bash
# Build do projeto
cargo build

# Build otimizado (release)
cargo build --release

# Executar
cargo run

# Executar com logs
RUST_LOG=debug cargo run

# Watch mode (recompila ao salvar)
cargo watch -x run

# Verificar código (linting)
cargo clippy

# Formatar código
cargo fmt

# Verificar formatação
cargo fmt --check

# Executar testes
cargo test

# Testes com output
cargo test -- --nocapture

# Testes de um módulo específico
cargo test api::product

# Verificar vulnerabilidades
cargo audit

# Limpar build artifacts
cargo clean

# Atualizar dependências
cargo update
```

---

## 🐳 Docker Compose

### Gerenciar Serviços

```bash
# Iniciar tudo
docker-compose up -d

# Parar tudo
docker-compose down

# Parar e remover volumes
docker-compose down -v

# Reiniciar serviço específico
docker-compose restart app

# Rebuild e restart
docker-compose up -d --build app

# Ver status
docker-compose ps

# Ver logs de todos os serviços
docker-compose logs -f

# Ver logs de um serviço
docker-compose logs -f app
docker-compose logs -f mongo

# Executa comando em um serviço
docker-compose exec app sh
docker-compose exec mongo mongosh
```

### Serviços Individuais

```bash
# App (Rust API)
docker-compose up -d app
docker-compose logs -f app
docker-compose restart app

# MongoDB
docker-compose up -d mongo
docker-compose exec mongo mongosh omnistore

# Redis
docker-compose up -d redis
docker-compose exec redis redis-cli

# MinIO
docker-compose up -d minio
# Console: http://localhost:9001
```

---

## 🗄️ MongoDB

### Comandos Úteis

```bash
# Conectar ao MongoDB
docker-compose exec mongo mongosh omnistore

# Ou via linha de comando direta
docker-compose exec mongo mongosh omnistore --eval "db.products.find().pretty()"
```

### Queries MongoDB

```javascript
// Listar coleções
show collections

// Contar documentos
db.products.countDocuments()
db.orders.countDocuments()

// Buscar todos
db.products.find()
db.products.find().pretty()
db.products.find().limit(5)

// Buscar por filtro
db.products.find({ active: true })
db.products.find({ name: "Produto X" })
db.products.find({ "inventory.quantity": { $gt: 0 } })

// Buscar um
db.products.findOne({ sku: "ABC123" })

// Buscar por ID
db.products.findOne({ _id: ObjectId("507f1f77bcf86cd799439011") })

// Atualizar
db.products.updateOne(
  { sku: "ABC123" },
  { $set: { active: false } }
)

// Deletar
db.products.deleteOne({ sku: "ABC123" })

// Agregar
db.orders.aggregate([
  { $match: { payment_status: "paid" } },
  { $group: { _id: null, total: { $sum: "$total" } } }
])

// Criar índice
db.products.createIndex({ sku: 1 }, { unique: true })
db.products.createIndex({ name: "text" })

// Ver índices
db.products.getIndexes()

// Dropar coleção (cuidado!)
db.products.drop()

// Limpar todos os documentos (cuidado!)
db.products.deleteMany({})

// Backup
mongodump --db omnistore --out /backup

// Restore
mongorestore --db omnistore /backup/omnistore
```

---

## 🔴 Redis

### Comandos Úteis

```bash
# Conectar ao Redis
docker-compose exec redis redis-cli

# Ou via comando direto
docker-compose exec redis redis-cli KEYS "*"
```

### Comandos Redis

```bash
# Listar todas as chaves
KEYS *

# Listar chaves por padrão
KEYS category:*
KEYS collection:*:products:*

# Ver valor de uma chave
GET product:123

# Ver tipo de uma chave
TYPE category:abc123:products:p1:l20

# Ver TTL
TTL category:abc123:products:p1:l20

# Deletar chave
DEL product:123

# Deletar por padrão (cuidado!)
# Redis não tem del pattern nativo, use SCAN
SCAN 0 MATCH category:* COUNT 100

# Limpar tudo (cuidado!)
FLUSHALL

# Info do servidor
INFO
INFO stats
INFO memory

# Monitor (ver comandos em tempo real)
MONITOR
```

---

## 📦 MinIO / S3

### Comandos Úteis

```bash
# Acessar MinIO Console
# http://localhost:9001
# User: minioadmin
# Pass: minioadmin (ou conforme .env)

# Usar mc (MinIO Client) - precisa instalar
# Instalar mc
brew install minio/stable/mc  # macOS
# ou baixe de https://min.io/download

# Configurar alias
mc alias set local http://localhost:9000 minioadmin minioadmin

# Listar buckets
mc ls local

# Listar arquivos
mc ls local/omnistore
mc ls local/omnistore/products/
mc ls local/omnistore/temp/

# Fazer upload
mc cp foto.jpg local/omnistore/products/

# Fazer download
mc cp local/omnistore/products/foto.jpg ./

# Deletar arquivo
mc rm local/omnistore/products/foto.jpg

# Deletar pasta recursivo
mc rm --recursive --force local/omnistore/temp/

# Criar bucket
mc mb local/omnistore

# Ver políticas do bucket
mc admin policy list local
```

---

## 🌐 API Requests (curl)

### Health Check

```bash
curl http://localhost:8080
curl http://localhost:8080/hello
```

### Products

```bash
# Listar produtos
curl http://localhost:8080/api/products

# Com paginação
curl "http://localhost:8080/api/products?page=1&limit=10"

# Com filtros
curl "http://localhost:8080/api/products?name=Camiseta"
curl "http://localhost:8080/api/products?category_id=507f1f77bcf86cd799439011"

# Buscar por ID
curl http://localhost:8080/api/products/507f1f77bcf86cd799439011

# Criar produto
curl -X POST http://localhost:8080/api/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Produto Teste",
    "sku": "TEST-001",
    "active": true
  }'

# Atualizar produto
curl -X PUT http://localhost:8080/api/products/507f1f77bcf86cd799439011 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Produto Atualizado",
    "active": false
  }'

# Deletar produto
curl -X DELETE http://localhost:8080/api/products/507f1f77bcf86cd799439011
```

### Orders

```bash
# Listar pedidos
curl http://localhost:8080/api/orders

# Com filtros
curl "http://localhost:8080/api/orders?customer_id=507f1f77bcf86cd799439011"
curl "http://localhost:8080/api/orders?payment_status=paid"

# Buscar por ID
curl http://localhost:8080/api/orders/507f1f77bcf86cd799439011

# Criar checkout (Mercado Pago)
curl -X POST http://localhost:8080/api/orders/checkout \
  -H "Content-Type: application/json" \
  -d '{
    "cart_id": "507f1f77bcf86cd799439011",
    "payer": {
      "email": "test@example.com"
    }
  }'

# Criar checkout (Asaas)
curl -X POST http://localhost:8080/api/orders/checkout/asaas \
  -H "Content-Type: application/json" \
  -d '{
    "cart_id": "507f1f77bcf86cd799439011",
    "customer": {
      "name": "João Silva",
      "cpfCnpj": "12345678901",
      "email": "joao@example.com"
    }
  }'
```

### Admin

```bash
# Dashboard (precisa de token)
curl http://localhost:8080/api/admin/dashboard \
  -H "X-Admin-Token: changeme"

# Listar pedidos (admin)
curl http://localhost:8080/api/admin/orders \
  -H "X-Admin-Token: changeme"

# Com filtros
curl "http://localhost:8080/api/admin/orders?payment_status=paid&page=1&limit=10" \
  -H "X-Admin-Token: changeme"
```

### Uploads

```bash
# Solicitar presigned URL
curl -X POST http://localhost:8080/api/uploads/presign \
  -H "Content-Type: application/json" \
  -d '{"filename": "foto.jpg"}'

# Fazer upload (usando presigned URL retornada)
curl -X PUT "<presigned_url>" \
  --upload-file foto.jpg

# Confirmar upload
curl -X POST http://localhost:8080/api/uploads/confirm \
  -H "Content-Type: application/json" \
  -d '{
    "temp_key": "temp/uuid-foto.jpg",
    "dest_folder": "products"
  }'

# Deletar arquivo
curl -X DELETE http://localhost:8080/api/uploads \
  -H "Content-Type: application/json" \
  -d '{"key": "products/foto.jpg"}'
```

---

## 🧪 Testing

### API Testing com httpie (alternativa ao curl)

```bash
# Instalar httpie
brew install httpie  # macOS
# ou pip install httpie

# Listar produtos (mais legível que curl)
http GET http://localhost:8080/api/products

# Criar produto
http POST http://localhost:8080/api/products \
  name="Produto Teste" \
  sku="TEST-001" \
  active:=true

# Com autenticação
http GET http://localhost:8080/api/admin/dashboard \
  X-Admin-Token:changeme
```

---

## 📊 Monitoring & Logs

### Ver Logs

```bash
# Logs da aplicação
docker-compose logs -f app

# Últimas 100 linhas
docker-compose logs --tail=100 app

# Desde horário específico
docker-compose logs --since 2024-04-14T10:00:00 app

# Grep nos logs
docker-compose logs app | grep ERROR
docker-compose logs app | grep "POST /api/orders"
```

### Métricas do Sistema

```bash
# Stats dos containers
docker stats

# Stats de um container específico
docker stats omnistore-app

# Uso de disco
docker system df

# Limpar recursos não usados
docker system prune -a
```

---

## 🔧 Troubleshooting

### Problemas Comuns

#### App não inicia

```bash
# Ver logs detalhados
docker-compose logs app

# Verificar variáveis de ambiente
docker-compose exec app env | grep MONGODB

# Rebuild completo
docker-compose down
docker-compose build --no-cache app
docker-compose up -d
```

#### MongoDB não conecta

```bash
# Verificar se está rodando
docker-compose ps mongo

# Ver logs
docker-compose logs mongo

# Testar conexão
docker-compose exec mongo mongosh --eval "db.adminCommand('ping')"

# Reiniciar
docker-compose restart mongo
```

#### Redis não conecta

```bash
# Verificar se está rodando
docker-compose ps redis

# Ver logs
docker-compose logs redis

# Testar conexão
docker-compose exec redis redis-cli ping

# Reiniciar
docker-compose restart redis
```

#### Erro de compilação Rust

```bash
# Limpar cache
cargo clean

# Atualizar dependências
cargo update

# Rebuild do zero
rm -rf target
cargo build
```

#### Porta já em uso

```bash
# Verificar o que está usando a porta 8080
lsof -i :8080

# Matar processo
kill -9 <PID>

# Ou mudar a porta no docker-compose.yaml
ports:
  - "8081:8080"  # muda de 8080 para 8081
```

---

## 📝 Git Workflow

```bash
# Criar branch para feature
git checkout -b feature/nome-da-feature

# Commit frequente
git add .
git commit -m "feat: adiciona validação de estoque"

# Push para remote
git push origin feature/nome-da-feature

# Atualizar da main
git checkout main
git pull
git checkout feature/nome-da-feature
git merge main

# Squash commits antes de PR
git rebase -i HEAD~3
```

---

## 🎨 Utilitários

### Gerar ObjectId MongoDB

```javascript
// No mongosh
ObjectId()

// Ou online
// https://observablehq.com/@hugodf/mongodb-objectid-generator
```

### Gerar UUID

```bash
# Linux/macOS
uuidgen

# Rust
use uuid::Uuid;
let id = Uuid::new_v4();
```

### Timestamp atual

```bash
# Unix timestamp
date +%s

# ISO 8601
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

### Base64 encode/decode

```bash
# Encode
echo "senha123" | base64

# Decode
echo "c2VuaGExMjM=" | base64 -d
```

---

## 📚 Recursos Úteis

### Documentação Oficial

- [Rust](https://doc.rust-lang.org/book/)
- [Axum](https://docs.rs/axum/)
- [MongoDB Rust Driver](https://docs.rs/mongodb/)
- [Redis Rust](https://docs.rs/redis/)
- [tokio](https://tokio.rs/)
- [serde](https://serde.rs/)

### Tools

- [Postman](https://www.postman.com/) - API testing
- [MongoDB Compass](https://www.mongodb.com/products/compass) - GUI do MongoDB
- [RedisInsight](https://redis.com/redis-enterprise/redis-insight/) - GUI do Redis
- [httpie](https://httpie.io/) - Alternativa ao curl

### Online Tools

- [MongoDB Playground](https://mongoplayground.net/)
- [JSON Formatter](https://jsonformatter.org/)
- [ObjectId Generator](https://observablehq.com/@hugodf/mongodb-objectid-generator)
- [Regex101](https://regex101.com/)

---

## 🆘 Ajuda

### Documentação do Projeto

- [README.md](README.md)
- [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)
- [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md)
- [CHECKLIST.md](CHECKLIST.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [DOCS_INDEX.md](DOCS_INDEX.md)

### Precisa de Ajuda?

1. Procure nesta referência rápida
2. Consulte a documentação do projeto
3. Pergunte no Slack #dev
4. Abra uma issue no GitHub

---

**Última atualização**: Abril 2026  
**Versão**: 1.0.0

