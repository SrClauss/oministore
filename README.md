# OmniStore

**E-commerce Headless API** para MEIs - Simples, rápido e leve.

API de e-commerce headless desenvolvida em Rust com Axum, MongoDB, Redis e MinIO. Focada em **simplicidade** e **performance** para rodar em uma única instância de servidor.

> **Status do Projeto**: 🟡 Em Desenvolvimento (60% completo)  
> **Versão**: 0.1.0  
> **Tipo**: Headless API (Single-tenant)  
> **Última Atualização**: Abril 2026

## 📚 Documentação

- **[📖 DOCS_INDEX.md](DOCS_INDEX.md)** - Índice e guia de navegação dos documentos
- **[⚡ QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Referência rápida de comandos
- **[📋 README.md](README.md)** - Este arquivo (Visão geral e setup)
- **[📊 EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)** - Resumo executivo e status
- **[🔍 DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md)** - Status detalhado por componente
- **[✅ CHECKLIST.md](CHECKLIST.md)** - Checklist de tarefas pendentes
- **[🏗️ ARCHITECTURE.md](ARCHITECTURE.md)** - Diagramas de arquitetura

> 💡 **Novo no projeto?** Comece com [DOCS_INDEX.md](DOCS_INDEX.md) para saber por onde começar!

## 📋 Visão Geral

OmniStore é uma **API headless de e-commerce** projetada para **MEIs e pequenos negócios**. Fornece todos os recursos de backend necessários (produtos, pedidos, estoque, pagamentos) via REST API, permitindo que você construa seu frontend customizado da forma que preferir.

### 🎯 Para Quem é Este Projeto?

- ✅ **MEIs** que querem uma loja online simples e profissional
- ✅ **Desenvolvedores** que precisam de um backend de e-commerce pronto
- ✅ **Negócios pequenos** que querem customização total do frontend
- ✅ **Projetos** que precisam rodar em um **único servidor VPS** (sem Kubernetes)

### 💡 Conceito Headless

Este é um **headless commerce** - apenas o backend/API. Você cria seu próprio frontend em:
- React, Vue, Next.js (web)
- React Native, Flutter (mobile)
- HTML/CSS/JS puro
- Ou qualquer tecnologia que consuma APIs REST

### Stack Tecnológica

- **Framework**: Axum 0.7 (Web framework assíncrono em Rust)
- **Linguagem**: Rust (Edition 2024) - Rápido, seguro e eficiente
- **Banco de Dados**: MongoDB 2.5 - Flexível e escalável
- **Cache**: Redis 0.27 - Performance de leitura
- **Storage**: MinIO (S3-compatible) - Armazenamento de imagens
- **Runtime**: Tokio - Async/await nativo
- **Deploy**: Docker & Docker Compose - Um comando e está no ar

### 🚀 Características

- ⚡ **Performance**: Rust + async = respostas em milissegundos
- 🪶 **Leve**: Roda em VPS de 1GB RAM (DigitalOcean, Hetzner, etc)
- 🔌 **Headless**: API REST pura, frontend totalmente customizável
- 🎨 **Flexível**: MongoDB permite estruturas de dados adaptáveis
- 💳 **Pagamentos**: Mercado Pago e Asaas integrados
- 📦 **Single-tenant**: Uma loja por instância, simples e direto
- 🐳 **Fácil deploy**: Docker Compose, sem Kubernetes ou complexidades

## 🏗️ Estrutura do Projeto

```
omnistore/
├── src/
│   ├── main.rs              # Entry point da aplicação
│   ├── mod.rs               # Módulo raiz
│   ├── api/                 # Camada de endpoints HTTP
│   │   ├── mod.rs
│   │   ├── admin.rs         # ✅ Painel administrativo
│   │   ├── audit_log.rs     # ✅ Logs de auditoria
│   │   ├── cart.rs          # ✅ Carrinho de compras
│   │   ├── cart_item.rs     # ✅ Itens do carrinho
│   │   ├── category.rs      # ✅ Categorias de produtos
│   │   ├── collection.rs    # ✅ Coleções de produtos
│   │   ├── coupon.rs        # ✅ Cupons de desconto
│   │   ├── customer.rs      # ✅ Clientes
│   │   ├── inventory.rs     # ✅ Estoque
│   │   ├── order.rs         # ✅ Pedidos + Checkout (MP/Asaas)
│   │   ├── product.rs       # ✅ Produtos
│   │   ├── shipping.rs      # ✅ Envio/Frete
│   │   ├── store.rs         # ✅ Configuração da loja (single-tenant)
│   │   ├── upload.rs        # ✅ Upload de arquivos
│   │   ├── user.rs          # ✅ Usuários
│   │   ├── warehouse.rs     # ✅ Armazéns
│   │   └── webhooks.rs      # ✅ Webhooks (MP/Asaas)
│   ├── models/              # Modelos de dados
│   │   ├── mod.rs
│   │   ├── audit_log.rs     # ✅ Modelo de log de auditoria
│   │   ├── cart.rs          # ✅ Modelo de carrinho
│   │   ├── cart_item.rs     # ✅ Modelo de item do carrinho
│   │   ├── category.rs      # ✅ Modelo de categoria
│   │   ├── collection.rs    # ✅ Modelo de coleção
│   │   ├── coupon.rs        # ✅ Modelo de cupom
│   │   ├── customer.rs      # ✅ Modelo de cliente
│   │   ├── field.rs         # ✅ Metadata helper
│   │   ├── inventory.rs     # ✅ Modelo de estoque
│   │   ├── order.rs         # ✅ Modelo de pedido
│   │   ├── product.rs       # ✅ Modelo de produto
│   │   ├── shipping.rs      # ✅ Modelo de envio
│   │   ├── store.rs         # ✅ Configuração da loja
│   │   ├── user.rs          # ✅ Modelo de usuário
│   │   └── warehouse.rs     # ✅ Modelo de armazém
│   └── services/            # Camada de serviços
│       ├── mod.rs
│       ├── cache.rs         # ✅ Redis cache service
│       ├── mongo.rs         # ✅ MongoDB operations
│       └── storage.rs       # ✅ MinIO/S3 operations
├── Cargo.toml               # Dependências do projeto
├── docker-compose.yaml      # Infraestrutura containerizada
├── Dockerfile               # Build da aplicação
├── .env.example             # Exemplo de variáveis de ambiente
└── README.md                # Esta documentação

```

## ✅ Funcionalidades Implementadas

### Core do Sistema

- ✅ **Servidor HTTP** (Axum) rodando na porta 8080
- ✅ **Logging** estruturado com tracing
- ✅ **Middleware** de logging de requisições
- ✅ **Conexão MongoDB** com pool de conexões
- ✅ **Conexão Redis** para cache
- ✅ **MinIO/S3** para armazenamento de arquivos

### APIs Implementadas

#### 1. **Produtos** (`/api/products`)
- ✅ `GET /` - Listar produtos com paginação e filtros
- ✅ `GET /:id` - Buscar produto por ID
- ✅ `POST /` - Criar novo produto
- ✅ `PUT /:id` - Atualizar produto
- ✅ `DELETE /:id` - Deletar produto
- ✅ Cache Redis em listagens por categoria
- ✅ Invalidação de cache em updates/deletes

#### 2. **Configuração da Loja** (`/api/store`)
- ✅ `GET /` - Obter configurações da loja única
- ✅ `PUT /` - Atualizar configurações (nome, logo, cores, gateway keys)

#### 3. **Pedidos** (`/api/orders`)
- ✅ `GET /` - Listar pedidos com filtros
- ✅ `GET /:id` - Buscar pedido por ID
- ✅ `POST /` - Criar novo pedido
- ✅ `PUT /:id` - Atualizar pedido
- ✅ `DELETE /:id` - Deletar pedido
- ✅ `POST /checkout` - Criar checkout com Mercado Pago
- ✅ `POST /checkout/asaas` - Criar checkout com Asaas

#### 4. **Carrinho** (`/api/carts`)
- ✅ `GET /` - Listar carrinhos
- ✅ `GET /:id` - Buscar carrinho por ID
- ✅ `POST /` - Criar novo carrinho
- ✅ `PUT /:id` - Atualizar carrinho
- ✅ `DELETE /:id` - Deletar carrinho

#### 5. **Categorias** (`/api/categories`)
- ✅ CRUD completo de categorias
- ✅ Filtros por nome

#### 6. **Coleções** (`/api/collections`)
- ✅ CRUD completo de coleções
- ✅ `GET /:id/products` - Listar produtos da coleção (paginado + cache)

#### 7. **Clientes** (`/api/customers`)
- ✅ CRUD completo de clientes
- ✅ Suporte a endereços

#### 8. **Cupons** (`/api/coupons`)
- ✅ CRUD completo de cupons de desconto

#### 9. **Estoque** (`/api/inventory`)
- ✅ CRUD completo de inventário
- ✅ Gestão de quantidades e reservas

#### 10. **Armazéns** (`/api/warehouses`)
- ✅ CRUD completo de armazéns

#### 11. **Envio/Frete** (`/api/shipping`)
- ✅ CRUD completo de opções de envio

#### 12. **Usuários** (`/api/users`)
- ✅ CRUD completo de usuários
- ✅ Roles: Admin, Manager, Staff

#### 13. **Uploads** (`/api/uploads`)
- ✅ `POST /presign` - Gerar URL pré-assinada para upload
- ✅ `POST /confirm` - Confirmar upload (move de temp/ para pasta definitiva)
- ✅ `DELETE /` - Deletar arquivo
- ✅ `POST /bulk-delete` - Deletar múltiplos arquivos
- ✅ Lifecycle policy: arquivos em `temp/` expiram em 1 dia

#### 14. **Webhooks** (`/api/webhooks`)
- ✅ `POST /mercadopago` - Webhook do Mercado Pago
- ✅ `POST /asaas` - Webhook do Asaas
- ✅ Validação HMAC-SHA256 de assinaturas
- ✅ Processamento assíncrono de eventos de pagamento

#### 15. **Logs de Auditoria** (`/api/audit-logs`)
- ✅ CRUD completo de logs de auditoria

#### 16. **Painel Admin** (`/api/admin`)
- ✅ `GET /dashboard` - Dashboard com métricas
  - Total de pedidos
  - Pedidos pagos/pendentes
  - Total de clientes
  - Total de produtos
  - Receita total
- ✅ `GET /orders` - Listar pedidos (admin)
- ✅ `GET /products` - Listar produtos (admin)
- ✅ `GET /customers` - Listar clientes (admin)
- ✅ `GET /coupons` - Listar cupons (admin)
- ✅ Autenticação via `X-Admin-Token` header

### Serviços Implementados

#### Cache (Redis)
- ✅ `get(key)` - Buscar do cache
- ✅ `set(key, value)` - Armazenar no cache (TTL 5 min)
- ✅ `set_with_ttl(key, value, ttl)` - Armazenar com TTL customizado
- ✅ `del(key)` - Deletar chave
- ✅ `del_pattern(pattern)` - Deletar por padrão (SCAN seguro)

#### MongoDB
- ✅ `find_all(collection, filter)` - Buscar todos
- ✅ `find_one(collection, id)` - Buscar por ID
- ✅ `insert_one(collection, document)` - Inserir
- ✅ `update_one(collection, id, updates)` - Atualizar
- ✅ `delete_one(collection, id)` - Deletar
- ✅ `find_paginated(collection, filter, page, limit)` - Busca paginada
- ✅ `find_by_ids_paginated(collection, ids, page, limit)` - Busca por IDs com paginação

#### Storage (MinIO/S3)
- ✅ `ensure_bucket()` - Criar bucket e aplicar lifecycle policies
- ✅ `presigned_upload_url(filename)` - Gerar URL pré-assinada para PUT
- ✅ `confirm_upload(temp_key, dest_folder)` - Mover de temp/ para pasta definitiva
- ✅ `delete_object(key)` - Deletar arquivo
- ✅ `delete_objects(keys)` - Deletar múltiplos arquivos
- ✅ `public_url(key)` - Gerar URL pública do arquivo

### Integrações de Pagamento

#### Mercado Pago
- ✅ Criação de preferências de pagamento
- ✅ Geração de checkout URL (sandbox/production)
- ✅ Webhook com validação de assinatura HMAC-SHA256
- ✅ Processamento de notificações de pagamento
- ✅ Atualização automática de status do pedido

#### Asaas
- ✅ Criação de clientes
- ✅ Criação de cobranças (boleto, cartão, PIX)
- ✅ Webhook com validação de assinatura
- ✅ Processamento de notificações de pagamento
- ✅ Atualização automática de status do pedido

### Recursos de Segurança

- ✅ Validação HMAC-SHA256 em webhooks
- ✅ Token de autenticação admin
- ✅ Sanitização de inputs
- ✅ Lifecycle policies para arquivos temporários

## ⚠️ Funcionalidades Pendentes/Melhorias

### Segurança & Autenticação

- ❌ **Autenticação JWT** para usuários finais
- ❌ **Hash de senha** (bcrypt/argon2) - atualmente apenas campo password_hash
- ❌ **Middleware de autorização** baseado em roles
- ❌ **Rate limiting** para proteção contra ataques
- ❌ **CORS configurável**

### Funcionalidades de Produto

- ❌ **Variantes de produto** (tamanho, cor, etc.) - campo existe mas lógica não implementada
- ❌ **Busca full-text** de produtos (MongoDB text search ou Elasticsearch)
- ❌ **Filtros avançados** (faixa de preço, múltiplas categorias, etc.)
- ❌ **Reviews e ratings** de produtos
- ❌ **Wishlist** (lista de desejos)

### Carrinho & Checkout

- ❌ **Validação de estoque** ao adicionar no carrinho
- ❌ **Cálculo automático de frete** (integração Correios/transportadoras)
- ❌ **Aplicação de cupons** com validação de limites e expiração
- ❌ **Abandono de carrinho** - emails de recuperação
- ❌ **Guest checkout** - compra sem cadastro

### Pedidos & Pagamento

- ❌ **Rastreamento de pedido** em tempo real
- ❌ **Integração com transportadoras** para rastreio
- ❌ **Gestão de devoluções/trocas**
- ❌ **Faturamento automático** (emissão de NF-e)
- ❌ **Split de pagamento** para multi-vendedores
- ❌ **Pagamento parcelado** com juros
- ❌ **Assinaturas/Pagamentos recorrentes**

### Notificações

- ❌ **Email transacional** (confirmação pedido, status de envio, etc.)
- ❌ **SMS** para notificações críticas
- ❌ **Push notifications** para app mobile
- ❌ **Webhook outbound** para integrações externas

### Analytics & Relatórios

- ❌ **Dashboard detalhado** (gráficos, tendências)
- ❌ **Relatórios de vendas** (diário, mensal, anual)
- ❌ **Relatório de produtos mais vendidos**
- ❌ **Relatório de abandono de carrinho**
- ❌ **Export de dados** (CSV, Excel)

### Internacionalização (Opcional)

- ❌ **Multi-moeda** (R$, USD, EUR)
- ❌ **Multi-idioma** (PT-BR, EN, ES)

### DevOps & Infraestrutura

- ❌ **CI/CD pipeline** (GitHub Actions simples)
- ❌ **Testes unitários** e de integração
- ❌ **Backup automático** de banco de dados
- ❌ **Health checks** avançados
- ❌ **Script de deploy** para VPS (deploy.sh)

### Performance

- ❌ **Cache de queries** mais agressivo
- ❌ **CDN** para imagens de produtos
- ❌ **Compressão de imagens** automática
- ❌ **LazyLoading** de dados relacionados
- ❌ **Database indexes** otimizados

### API & Documentação

- ❌ **OpenAPI/Swagger** documentation
- ❌ **GraphQL API** (alternativa ao REST)
- ❌ **SDK** para integração (JavaScript, Python, etc.)
- ❌ **Webhooks customizáveis**
- ❌ **API versioning** (v1, v2, etc.)

## 🚀 Configuração e Instalação

### Opção 1: Local (Desenvolvimento)

**Pré-requisitos:**
- Docker & Docker Compose
- Git

**Um comando para testar:**
```bash
git clone <repo>
cd omnistore
cp .env.example .env
docker-compose up -d
```

Pronto! API rodando em `http://localhost:8080`

### Opção 2: VPS (Produção)

**Requisitos mínimos:**
- VPS com 1GB RAM (DigitalOcean, Hetzner, Contabo)
- Ubuntu 22.04 ou similar
- Docker instalado

**Deploy:**
```bash
# No servidor
git clone <repo>
cd omnistore
cp .env.example .env
nano .env  # Configure suas variáveis
docker-compose up -d
```

### Variáveis de Ambiente

Copie o arquivo `.env.example` para `.env` e configure:

```bash
cp .env.example .env
```

Principais variáveis:

```env
# MongoDB
MONGODB_URI=mongodb://mongo:27017

# Redis
REDIS_URL=redis://redis:6379/0

# MinIO
MINIO_ENDPOINT=http://minio:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin

# Mercado Pago
MERCADO_PAGO_ACCESS_TOKEN=TEST-XXXXXXXX...
MERCADO_PAGO_WEBHOOK_SECRET=your_secret

# Asaas
ASAAS_API_KEY=$aact_hmlg_XXXXXXXX...
ASAAS_SANDBOX=true

# Admin
ADMIN_TOKEN=changeme
```

### Executar com Docker Compose

```bash
# Iniciar todos os serviços
docker-compose up -d

# Ver logs
docker-compose logs -f app

# Parar serviços
docker-compose down
```

### Executar em Desenvolvimento

```bash
# Instalar dependências e compilar
cargo build

# Executar
cargo run

# Executar em modo watch
cargo watch -x run
```

A API estará disponível em: `http://localhost:8080`

## 📡 Endpoints da API

### Públicos

- `GET /` - Health check
- `GET /hello` - Hello world

### Produtos

- `GET /api/products` - Listar produtos
  - Query params: `name`, `sku`, `category_id`, `page`, `limit`
- `GET /api/products/:id` - Buscar produto
- `POST /api/products` - Criar produto
- `PUT /api/products/:id` - Atualizar produto
- `DELETE /api/products/:id` - Deletar produto

### Configuração da Loja

- `GET /api/store` - Obter configurações da loja
- `PUT /api/store` - Atualizar configurações (nome, logo, tema, gateway keys)

### Pedidos

- `GET /api/orders` - Listar pedidos
- `GET /api/orders/:id` - Buscar pedido
- `POST /api/orders` - Criar pedido
- `PUT /api/orders/:id` - Atualizar pedido
- `DELETE /api/orders/:id` - Deletar pedido
- `POST /api/orders/checkout` - Criar checkout Mercado Pago
- `POST /api/orders/checkout/asaas` - Criar checkout Asaas

### Carrinho

- `GET /api/carts` - Listar carrinhos
- `GET /api/carts/:id` - Buscar carrinho
- `POST /api/carts` - Criar carrinho
- `PUT /api/carts/:id` - Atualizar carrinho
- `DELETE /api/carts/:id` - Deletar carrinho

### Uploads

- `POST /api/uploads/presign` - Gerar URL pré-assinada
- `POST /api/uploads/confirm` - Confirmar upload
- `DELETE /api/uploads` - Deletar arquivo
- `POST /api/uploads/bulk-delete` - Deletar múltiplos

### Webhooks

- `POST /api/webhooks/mercadopago` - Webhook Mercado Pago
- `POST /api/webhooks/asaas` - Webhook Asaas

### Admin (requer `X-Admin-Token` header)

- `GET /api/admin/dashboard` - Dashboard com métricas
- `GET /api/admin/orders` - Listar pedidos
- `GET /api/admin/products` - Listar produtos
- `GET /api/admin/customers` - Listar clientes
- `GET /api/admin/coupons` - Listar cupons

### Outros Recursos

Endpoints similares disponíveis para:
- `/api/categories` - Categorização de produtos
- `/api/collections` - Coleções/vitrines
- `/api/customers` - Cadastro de clientes
- `/api/coupons` - Cupons de desconto
- `/api/inventory` - Controle de estoque
- `/api/shipping` - Opções de frete
- `/api/users` - Usuários admin
- `/api/audit-logs` - Logs de auditoria
- `/api/audit-logs`

## 🧪 Testes

```bash
# Executar testes (quando implementados)
cargo test

# Executar com coverage
cargo tarpaulin --out Html
```

## 📦 Build para Produção

```bash
# Build otimizado
cargo build --release

# Executar binário
./target/release/omnistore
```

## 🤝 Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📝 Licença

Este projeto está sob a licença MIT.

## 🔗 Links Úteis

- [Axum Documentation](https://docs.rs/axum/)
- [MongoDB Rust Driver](https://docs.rs/mongodb/)
- [Redis Rust Client](https://docs.rs/redis/)
- [AWS SDK for Rust (S3)](https://docs.rs/aws-sdk-s3/)
- [Mercado Pago API](https://www.mercadopago.com.br/developers)
- [Asaas API](https://docs.asaas.com/)

---

**Status do Projeto**: 🟡 Em Desenvolvimento Ativo

**Última atualização**: Abril 2026
