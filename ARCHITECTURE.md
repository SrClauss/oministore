# Arquitetura do Sistema - OmniStore

Diagramas e documentação da arquitetura do sistema.

---

## 🏛️ Visão Geral da Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND                                 │
│                    (React / Vue / Next.js)                      │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/HTTPS
                         │ REST API
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                      LOAD BALANCER                               │
│                      (Nginx / AWS LB)                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    OMNISTORE API (Rust/Axum)                    │
│                         Port: 8080                               │
│                                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌─────────────────┐  │
│  │   API Layer    │  │  Services      │  │   Models        │  │
│  │                │  │                │  │                 │  │
│  │  - products    │  │  - mongo       │  │  - Product      │  │
│  │  - orders      │─▶│  - cache       │◀─│  - Order        │  │
│  │  - carts       │  │  - storage     │  │  - Cart         │  │
│  │  - customers   │  │                │  │  - Customer     │  │
│  │  - webhooks    │  └────────────────┘  │  - etc...       │  │
│  │  - admin       │                       │                 │  │
│  │  - uploads     │                       └─────────────────┘  │
│  └────────────────┘                                             │
└────────┬──────────┬──────────┬──────────────┬───────────────────┘
         │          │          │              │
         │          │          │              │
         ▼          ▼          ▼              ▼
    ┌────────┐ ┌───────┐ ┌─────────┐  ┌──────────────┐
    │MongoDB │ │ Redis │ │ MinIO   │  │   External   │
    │        │ │       │ │  (S3)   │  │   Services   │
    │ Port:  │ │ Port: │ │ Port:   │  │              │
    │ 27017  │ │ 6379  │ │ 9000    │  │ - MercadoPago│
    └────────┘ └───────┘ └─────────┘  │ - Asaas      │
                                       └──────────────┘
```

---

## 📦 Estrutura em Camadas

```
┌──────────────────────────────────────────────────────────────┐
│                        PRESENTATION                          │
│                      (HTTP Endpoints)                        │
│                                                              │
│  GET /api/products, POST /api/orders, etc.                  │
│  - Request parsing                                           │
│  - Response formatting (JSON)                                │
│  - Input validation                                          │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                     BUSINESS LOGIC                           │
│                    (API Handlers)                            │
│                                                              │
│  - Product management                                        │
│  - Order processing                                          │
│  - Cart operations                                           │
│  - Payment integration                                       │
│  - Authentication & Authorization                            │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                      SERVICE LAYER                           │
│                   (Shared Services)                          │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌─────────────┐              │
│  │  Mongo   │  │  Cache   │  │  Storage    │              │
│  │ Service  │  │ Service  │  │  Service    │              │
│  │          │  │          │  │             │              │
│  │ - CRUD   │  │ - Get    │  │ - Upload    │              │
│  │ - Find   │  │ - Set    │  │ - Delete    │              │
│  │ - Page   │  │ - Del    │  │ - Presign   │              │
│  └──────────┘  └──────────┘  └─────────────┘              │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                   DATA ACCESS LAYER                          │
│                   (Database Clients)                         │
│                                                              │
│  MongoDB Driver | Redis Client | AWS S3 SDK                 │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔄 Fluxo de Requisição

### Exemplo: Criar um Pedido

```
┌─────────┐
│ Cliente │
└────┬────┘
     │ 1. POST /api/orders/checkout
     │    { cart_id, customer, ... }
     ▼
┌─────────────────┐
│  API Gateway    │
│  (Middleware)   │
│                 │
│ - Log request   │
└────┬────────────┘
     │ 2. Route to handler
     ▼
┌──────────────────────────────────┐
│  Order API Handler               │
│  (src/api/order.rs)              │
│                                  │
│  create_order_checkout()         │
└────┬─────────────────────────────┘
     │ 3. Buscar carrinho
     ▼
┌──────────────────────┐
│  Mongo Service       │
│                      │
│  find_one("carts")   │
└────┬─────────────────┘
     │ 4. Carrinho encontrado
     ▼
┌──────────────────────────────────┐
│  Order API Handler               │
│                                  │
│  - Criar preferência MP          │
└────┬─────────────────────────────┘
     │ 5. HTTP POST
     ▼
┌──────────────────────┐
│  Mercado Pago API    │
│  (External)          │
│                      │
│  create_preference() │
└────┬─────────────────┘
     │ 6. { id, checkout_url }
     ▼
┌──────────────────────────────────┐
│  Order API Handler               │
│                                  │
│  - Criar pedido no banco         │
└────┬─────────────────────────────┘
     │ 7. insert_one
     ▼
┌──────────────────────┐
│  Mongo Service       │
│                      │
│  insert_one("orders")│
└────┬─────────────────┘
     │ 8. { inserted_id }
     ▼
┌──────────────────────────────────┐
│  Order API Handler               │
│                                  │
│  - Atualizar carrinho            │
└────┬─────────────────────────────┘
     │ 9. update_one
     ▼
┌──────────────────────┐
│  Mongo Service       │
│                      │
│  update("carts")     │
└────┬─────────────────┘
     │ 10. OK
     ▼
┌──────────────────────────────────┐
│  Order API Handler               │
│                                  │
│  return JSON response            │
└────┬─────────────────────────────┘
     │ 11. { order_id, checkout_url }
     ▼
┌─────────┐
│ Cliente │ → Redireciona para checkout_url
└─────────┘
```

---

## 🔐 Fluxo de Autenticação (Atual vs Planejado)

### Estado Atual

```
┌─────────┐
│ Cliente │
└────┬────┘
     │ Request (NO AUTH)
     ▼
┌──────────────┐
│  API Server  │  ← Sem validação
└──────────────┘

Exceção: Admin endpoints
┌─────────┐
│  Admin  │
└────┬────┘
     │ X-Admin-Token: secret
     ▼
┌──────────────┐
│  AdminAuth   │  → Valida token estático
│  Extractor   │
└──────────────┘
```

### Estado Planejado

```
┌─────────┐
│ Cliente │
└────┬────┘
     │ Authorization: Bearer <JWT>
     ▼
┌──────────────┐
│  API Server  │
│  Middleware  │
│              │
│  - Extract   │
│  - Verify    │
│  - Decode    │
└────┬─────────┘
     │ Valid JWT
     ▼
┌────────────────┐
│  Authorize     │
│  Check roles   │
│  Check perms   │
└────┬───────────┘
     │ Authorized
     ▼
┌──────────────┐
│  Handler     │
└──────────────┘
```

---

## 💾 Modelo de Dados

### Relacionamentos

```
┌─────────────┐
│   Store     │───────┐
└─────────────┘       │
                      │
                      │ has many
                      ▼
                 ┌──────────┐
         ┌───────│  Order   │
         │       └──────────┘
         │            │
         │            │ has many
         │            ▼
         │       ┌─────────────┐
         │       │  OrderItem  │
         │       └─────────────┘
         │            │
         │            │ references
         │            ▼
    has many    ┌──────────┐        belongs to many
         │      │ Product  │◄───────────────┐
         │      └──────────┘                │
         │           │                      │
         │           │ belongs to many      │
         │           ▼                      │
         │      ┌────────────┐              │
         └─────▶│  Category  │              │
                └────────────┘              │
                                            │
                     ┌──────────────┐       │
                     │  Collection  │───────┘
                     └──────────────┘


┌─────────────┐
│  Customer   │
└─────────────┘
       │
       │ has many
       ▼
┌─────────────┐
│    Cart     │
└─────────────┘
       │
       │ has many
       ▼
┌──────────────┐
│  CartItem    │───┐
└──────────────┘   │
                   │ references
                   ▼
              ┌──────────┐
              │ Product  │
              └──────────┘


┌──────────────┐
│   Product    │
└──────────────┘
       │
       │ has one
       ▼
┌──────────────┐
│  Inventory   │
└──────────────┘
       │
       │ references
       ▼
┌──────────────┐
│  Warehouse   │
└──────────────┘
```

### Estrutura de Coleções MongoDB

```
omnistore (database)
├── products
│   ├── _id: ObjectId
│   ├── name: String
│   ├── sku: String (unique)
│   ├── category_ids: [ObjectId]
│   ├── inventory: Embedded
│   └── ...
├── orders
│   ├── _id: ObjectId
│   ├── store_id: ObjectId
│   ├── customer_id: ObjectId
│   ├── items: [Embedded OrderItem]
│   └── ...
├── carts
│   ├── _id: ObjectId
│   ├── user_id: ObjectId
│   ├── items: [Embedded CartItem]
│   └── ...
├── stores
├── categories
├── collections
├── customers
├── coupons
├── users
├── warehouses
├── audit_logs
└── ...
```

---

## 📊 Estratégia de Cache (Redis)

### Cache Keys Pattern

```
# Produtos por categoria (TTL: 5min)
category:{category_id}:products:p{page}:l{limit}
Exemplo: category:507f1f77bcf86cd799439011:products:p1:l20

# Produtos de coleção (TTL: 5min)
collection:{collection_id}:products:p{page}:l{limit}
Exemplo: collection:507f1f77bcf86cd799439012:products:p1:l20

# Produto individual (futuro)
product:{product_id}
Exemplo: product:507f1f77bcf86cd799439013

# Carrinho do usuário (futuro)
cart:user:{user_id}
Exemplo: cart:user:507f1f77bcf86cd799439014
```

### Cache Invalidation

```
Ao atualizar/deletar produto:
  → del_pattern("category:*:products:*")
  → del_pattern("collection:*:products:*")

Ao deletar coleção:
  → del_pattern("collection:{id}:products:*")

Ao atualizar carrinho:
  → del("cart:user:{user_id}")
```

### Cache Flow

```
┌─────────┐
│ Request │
└────┬────┘
     │
     ▼
┌──────────────┐
│ Check Cache  │
└────┬─────────┘
     │
     ├─── Hit ────▶ Return cached data
     │
     └─── Miss ───▶ ┌──────────────┐
                    │ Query DB     │
                    └────┬─────────┘
                         │
                         ▼
                    ┌──────────────┐
                    │ Store Cache  │
                    │ (TTL 5min)   │
                    └────┬─────────┘
                         │
                         ▼
                    Return fresh data
```

---

## 📁 Sistema de Arquivos (MinIO)

### Bucket Structure

```
omnistore (bucket)
├── temp/
│   ├── {uuid}-filename.jpg    ← Expira em 1 dia
│   ├── {uuid}-filename.png
│   └── ...
├── products/
│   ├── product-123.jpg
│   ├── product-123-2.jpg
│   └── ...
├── categories/
│   └── category-icon.svg
├── stores/
│   └── store-logo.png
├── users/
│   └── avatar.jpg
└── ...
```

### Upload Flow

```
┌─────────┐
│Frontend │
└────┬────┘
     │ 1. POST /api/uploads/presign
     │    { filename: "foto.jpg" }
     ▼
┌──────────────┐
│   Backend    │
└────┬─────────┘
     │ 2. Generate presigned URL
     │    key: temp/{uuid}-foto.jpg
     ▼
┌─────────┐
│ MinIO   │
└────┬────┘
     │ 3. Return presigned URL (15min TTL)
     ▼
┌─────────┐
│Frontend │
└────┬────┘
     │ 4. PUT to presigned URL (binary)
     ▼
┌─────────┐
│ MinIO   │ ← File stored in temp/
└─────────┘
     │ 5. Success
     ▼
┌─────────┐
│Frontend │
└────┬────┘
     │ 6. POST /api/products
     │    { photos: ["temp/{uuid}-foto.jpg"] }
     ▼
┌──────────────┐
│   Backend    │
└────┬─────────┘
     │ 7. Insert product
     ▼
┌─────────┐
│ MongoDB │
└─────────┘
     │ 8. Success
     ▼
┌─────────┐
│Frontend │
└────┬────┘
     │ 9. POST /api/uploads/confirm
     │    { temp_key: "temp/...", dest_folder: "products" }
     ▼
┌──────────────┐
│   Backend    │
└────┬─────────┘
     │ 10. Copy temp → products/foto.jpg
     │ 11. Delete temp/{uuid}-foto.jpg
     ▼
┌─────────┐
│ MinIO   │
└─────────┘
     │ 12. Return final key
     ▼
┌─────────┐
│Frontend │
└────┬────┘
     │ 13. PUT /api/products/:id
     │     { photos: ["products/foto.jpg"] }
     ▼
Done!
```

---

## 🎯 Processamento de Webhooks

### Mercado Pago Webhook Flow

```
┌──────────────┐
│ Mercado Pago │
└──────┬───────┘
       │ POST /api/webhooks/mercadopago
       │ X-Signature: ts=123,v1=abc123...
       │ { action: "payment.updated", data: { id: "123" } }
       ▼
┌────────────────────┐
│  Webhook Handler   │
└─────┬──────────────┘
      │ 1. Extract signature
      │ 2. Compute HMAC-SHA256
      │ 3. Compare signatures
      ▼
┌────────────────────┐
│  Valid?            │
└─────┬──────────────┘
      │
      ├─── No ────▶ Return 401 Unauthorized
      │
      └─── Yes ───▶ ┌────────────────────┐
                    │ Return 200 OK      │
                    │ (acknowledge fast) │
                    └─────┬──────────────┘
                          │
                          ▼
                    ┌─────────────────────────┐
                    │ Process Asynchronously  │
                    │                         │
                    │ - Fetch payment from MP │
                    │ - Find order by ref     │
                    │ - Update payment status │
                    │ - Update order status   │
                    │ - Send confirmation     │
                    └─────────────────────────┘
```

### Asaas Webhook Flow

```
┌──────────────┐
│    Asaas     │
└──────┬───────┘
       │ POST /api/webhooks/asaas
       │ asaas-access-token: webhook_secret
       │ { event: "PAYMENT_RECEIVED", payment: {...} }
       ▼
┌────────────────────┐
│  Webhook Handler   │
└─────┬──────────────┘
      │ 1. Extract token header
      │ 2. Compare with secret
      ▼
┌────────────────────┐
│  Valid?            │
└─────┬──────────────┘
      │
      ├─── No ────▶ Return 401 Unauthorized
      │
      └─── Yes ───▶ ┌────────────────────┐
                    │ Return 200 OK      │
                    └─────┬──────────────┘
                          │
                          ▼
                    ┌─────────────────────────┐
                    │ Process Event           │
                    │                         │
                    │ - PAYMENT_RECEIVED      │
                    │ - PAYMENT_CONFIRMED     │
                    │ - PAYMENT_DELETED       │
                    │ - etc.                  │
                    └─────────────────────────┘
```

---

## 🌐 Deployment Architecture (Planejado)

### Production Setup

```
                         ┌─────────────┐
                         │   Route 53  │ (DNS)
                         └──────┬──────┘
                                │
                                ▼
                         ┌─────────────┐
                         │  CloudFront │ (CDN)
                         └──────┬──────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
                ▼               ▼               ▼
         ┌───────────┐   ┌───────────┐   ┌───────────┐
         │   ALB     │   │   ALB     │   │   ALB     │
         │ (us-east) │   │ (eu-west) │   │ (sa-east) │
         └─────┬─────┘   └─────┬─────┘   └─────┬─────┘
               │               │               │
         ┌─────┴─────┐   ┌─────┴─────┐   ┌─────┴─────┐
         ▼           ▼   ▼           ▼   ▼           ▼
     ┌─────┐     ┌─────┐ ┌─────┐     ┌─────┐     ┌─────┐
     │ ECS │     │ ECS │ │ ECS │     │ ECS │     │ ECS │
     │ API │     │ API │ │ API │     │ API │     │ API │
     └──┬──┘     └──┬──┘ └──┬──┘     └──┬──┘     └──┬──┘
        │           │       │           │           │
        └───────────┴───────┴───────────┴───────────┘
                            │
                ┌───────────┼───────────┐
                │           │           │
                ▼           ▼           ▼
         ┌──────────┐ ┌─────────┐ ┌──────────┐
         │ MongoDB  │ │  Redis  │ │   S3     │
         │ Atlas    │ │Elasticac│ │ Bucket   │
         │(Replica) │ │  (Clust)│ │          │
         └──────────┘ └─────────┘ └──────────┘
```

### Docker Compose (Development)

```
┌────────────────────────────────────────┐
│         Docker Compose Network         │
│                                        │
│  ┌──────────────┐                     │
│  │   App        │  Port: 8080         │
│  │  (Rust/Axum) │                     │
│  └───┬──────────┘                     │
│      │                                 │
│  ┌───▼──────┬───────────┬──────────┐  │
│  │          │           │          │  │
│  ▼          ▼           ▼          ▼  │
│ ┌────┐  ┌─────┐  ┌───────┐  ┌───────┐│
│ │Mongo│  │Redis│  │ MinIO │  │MinIO  ││
│ │     │  │     │  │       │  │Console││
│ │27018│  │6379 │  │ 9000  │  │ 9001  ││
│ └────┘  └─────┘  └───────┘  └───────┘│
│                                        │
└────────────────────────────────────────┘
         All on same network
         Volumes for persistence
```

---

## 🔄 Continuous Integration/Deployment (Futuro)

```
┌─────────┐
│  Push   │
│  Code   │
└────┬────┘
     │
     ▼
┌──────────────────┐
│  GitHub Actions  │
└────┬─────────────┘
     │
     ├─▶ Run Tests (cargo test)
     │
     ├─▶ Lint (cargo clippy)
     │
     ├─▶ Format check (cargo fmt)
     │
     ├─▶ Security audit (cargo audit)
     │
     └─▶ Build Docker image
         │
         ▼
    ┌─────────────┐
    │ Push to ECR │
    └────┬────────┘
         │
         ▼
    ┌──────────────┐
    │ Deploy to    │
    │ Staging      │
    └────┬─────────┘
         │
         ▼
    ┌──────────────┐
    │ Run E2E      │
    │ Tests        │
    └────┬─────────┘
         │
         ├─── Pass ───▶ ┌──────────────┐
         │              │ Deploy to    │
         │              │ Production   │
         │              └──────────────┘
         │
         └─── Fail ───▶ ┌──────────────┐
                        │ Rollback     │
                        │ Notify Team  │
                        └──────────────┘
```

---

## 📊 Monitoring & Observability (Futuro)

```
┌────────────┐
│ API Server │
└─────┬──────┘
      │ Logs
      │ Metrics
      │ Traces
      │
      ├────────▶ ┌──────────────┐
      │          │ Prometheus   │ (Metrics)
      │          └──────┬───────┘
      │                 │
      │                 ▼
      │          ┌──────────────┐
      │          │  Grafana     │ (Dashboards)
      │          └──────────────┘
      │
      ├────────▶ ┌──────────────┐
      │          │ Loki         │ (Logs)
      │          └──────────────┘
      │
      └────────▶ ┌──────────────┐
                 │ Jaeger       │ (Traces)
                 └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │ Alertmanager │
                 └──────┬───────┘
                        │
                        ▼
                 ┌──────────────┐
                 │ Slack/Email  │
                 │ PagerDuty    │
                 └──────────────┘
```

---

**Última atualização**: Abril 2026  
**Versão** 1.0.0

