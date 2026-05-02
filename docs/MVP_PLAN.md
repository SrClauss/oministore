# Plano de Implementação do MVP — OmniStore

> **Contexto**: Backend headless em Rust (~65% implementado). Foco em MEI e pequenos lojistas, site de baixo volume de acessos.

---

## 🎯 Objetivo do MVP

Ter uma loja online funcionando com:

- Catálogo de produtos navegável
- Carrinho e checkout com pagamento real (Mercado Pago ou Asaas)
- Gestão mínima pelo lojista (admin)
- Deploy simples em VPS de baixo custo

---

## 📊 Estado Atual do Backend

| Área | Status | Pronto para MVP? |
|------|--------|-----------------|
| CRUD Produtos/Categorias | ✅ 100% | ✅ Sim |
| Carrinho | ✅ 100% | ⚠️ Parcial (falta validar estoque) |
| Pedidos + Checkout MP/Asaas | ✅ 100% | ⚠️ Parcial (falta decrementar estoque) |
| Webhooks de Pagamento | ✅ 80% | ✅ Sim |
| Upload de Imagens | ✅ 90% | ✅ Sim |
| Admin Panel (API) | ✅ 40% | ✅ Sim (suficiente para MVP) |
| Autenticação de Clientes | ❌ 0% | ❌ **Bloqueador** |
| CORS | ❌ 0% | ❌ **Bloqueador** |
| Estoque (decrementar ao vender) | ❌ 0% | ❌ **Bloqueador** |
| Email de confirmação | ❌ 0% | ⚠️ Recomendado |
| HTTPS / Deploy | ❌ 0% | ❌ **Bloqueador** |

---

## 🚧 Fases do MVP

### Fase 1 — Desbloqueadores Críticos (Backend)

> Sem esses itens, a loja não pode operar com segurança.

#### 1.1 CORS Configurável

**O que fazer:**
- Adicionar middleware de CORS no Axum com origem configurável via `.env`
- Variável `CORS_ORIGINS=https://minha-loja.com.br,http://localhost:3000`
- Liberar origens específicas em vez de `*` (exceto em dev)

**Arquivo**: `src/main.rs` + `.env.example`

---

#### 1.2 Autenticação de Clientes via Firebase Auth

> **Por que Firebase?** Gratuito, lida com login social (Google), recuperação de senha, SMS — sem backend próprio de auth.

**O que fazer:**
1. No backend: middleware que recebe o `Authorization: Bearer <firebase_id_token>` e valida com a chave pública do Firebase
2. Extrair `uid` do token e associar ao modelo `Customer`
3. Endpoint `GET /api/me` — retorna o customer logado
4. Endpoint `POST /api/me/register` — cria o Customer na primeira vez após login no Firebase

**Variáveis de ambiente necessárias:**
```env
FIREBASE_PROJECT_ID=meu-projeto-firebase
```

**Dependências a adicionar no `Cargo.toml`:**
- `jsonwebtoken` (verificação de JWT RS256)

**Rotas protegidas no MVP:**
- `POST /api/carts` — criar carrinho
- `POST /api/orders` — criar pedido
- `GET /api/me` — perfil do cliente

**Rotas públicas (sem autenticação):**
- `GET /api/products` — listar produtos
- `GET /api/products/:id` — detalhe do produto
- `GET /api/categories` — categorias
- `GET /api/store` — configurações da loja (nome, tema, etc.)

---

#### 1.3 Decrementar Estoque ao Confirmar Pagamento

**Onde implementar**: `src/api/webhooks.rs` — nos handlers de `mercadopago` e `asaas`

**Lógica:**
1. Ao receber webhook de pagamento aprovado:
   - Buscar o pedido pelo external_reference
   - Para cada item do pedido, decrementar `quantity` no `Inventory`
   - Se `quantity <= 0` e `sell_without_stock == false`, marcar produto como indisponível
2. Ao receber webhook de pagamento cancelado/expirado:
   - Não decrementar (manter estoque)

---

#### 1.4 Deploy: HTTPS + Nginx + VPS

**O que fazer:**
1. Criar `nginx.conf` na raiz do projeto com reverse proxy para `localhost:8080`
2. Configurar Let's Encrypt (Certbot) para SSL automático
3. Criar script `scripts/deploy.sh` de instalação no servidor

**Requisitos mínimos de VPS:**
- 1 vCPU, 1 GB RAM (ex: Hetzner CX11 ~€4/mês, Contabo VPS S ~€4/mês)
- Ubuntu 22.04 LTS
- Docker + Docker Compose instalado

**Script de deploy mínimo:**
```bash
#!/bin/bash
git pull origin main
docker-compose down
docker-compose up -d --build
```

---

### Fase 2 — Qualidade de Vida (Recomendado antes do Soft Launch)

#### 2.1 Email de Confirmação de Pedido

**Abordagem recomendada**: [Resend](https://resend.com) — gratuito até 3.000 emails/mês, API simples REST.

**O que fazer:**
- Ao criar pedido confirmado (webhook de pagamento aprovado), enviar email com:
  - Número do pedido
  - Itens comprados
  - Valor total
  - Estimativa de entrega (se aplicável)

**Variáveis necessárias:**
```env
RESEND_API_KEY=re_xxxxxxxxx
STORE_EMAIL=loja@meudominio.com.br
```

---

#### 2.2 Validação Básica de Cupons no Checkout

**O que fazer:**
- Endpoint `POST /api/coupons/validate` recebe `{ code, cart_total }` e retorna `{ valid, discount_value, discount_type }`
- Validar: código existe, não expirou, ainda tem usos disponíveis
- Ao criar o pedido, aplicar o desconto e incrementar `usage_count` do cupom

---

#### 2.3 Health Check Endpoint

**O que fazer:**
- Adicionar `GET /health` que retorna `200 OK` com status dos serviços (MongoDB, Redis, MinIO)
- Necessário para monitoramento básico e para o Nginx saber se o app está vivo

```json
{
  "status": "ok",
  "mongo": "ok",
  "redis": "ok",
  "storage": "ok"
}
```

---

### Fase 3 — Frontend

> O backend é headless. É necessário construir ou usar um frontend que consuma a API.

#### Opções recomendadas para MEI (ordem de velocidade de entrega)

| Opção | Esforço | Custo | Adequado para |
|-------|---------|-------|---------------|
| **Next.js + template pronto** | ⚡ Baixo | Gratuito | MVP rápido |
| **Astro.js + componentes** | Médio | Gratuito | SEO forte |
| **React SPA pura** | Médio | Gratuito | App interativo |
| **Shopify/Nuvemshop (externo)** | ⚡ Muito baixo | Pago | Quer zero código |

**Recomendação para MVP**: **Next.js App Router**
- SSR nativo (bom para SEO de produtos)
- Vercel gratuito para deploy do frontend
- Componentes de UI prontos (shadcn/ui ou Tailwind UI)
- Firebase Auth SDK fácil de integrar

**Páginas mínimas do frontend:**
1. `/` — Home com vitrine de produtos em destaque
2. `/produtos` — Catálogo com filtros por categoria
3. `/produtos/[id]` — Detalhe do produto
4. `/carrinho` — Carrinho de compras
5. `/checkout` — Formulário + gateway de pagamento
6. `/pedidos` — Histórico de pedidos do cliente logado
7. `/login` — Login via Firebase (Google + email/senha)

---

## 🗓️ Cronograma Sugerido

```
Semana 1: Fase 1 (Backend Desbloqueadores)
├── Dia 1-2: CORS + Firebase Auth middleware
├── Dia 3-4: Decrementar estoque no webhook
└── Dia 5:   Health check + ajustes finais de configuração

Semana 2: Fase 1 (Deploy)
├── Dia 1-2: nginx.conf + script de deploy
├── Dia 3:   Provisionar VPS + DNS + SSL
└── Dia 4-5: Testes de ponta a ponta em staging

Semana 3-4: Fase 2 (Qualidade)
├── Email de confirmação
├── Validação de cupons
└── Ajustes de bug

Semana 5-8: Fase 3 (Frontend)
├── Estrutura Next.js + integração Firebase
├── Páginas de catálogo e produto
├── Carrinho + Checkout
└── Área do cliente
```

---

## 💸 Custos Estimados de Infraestrutura

| Serviço | Plano | Custo/mês |
|---------|-------|-----------|
| VPS (ex: Hetzner CX22) | 2vCPU/4GB RAM | ~€6 |
| Domínio (.com.br) | Registro.br | ~R$40/ano |
| SSL (Let's Encrypt) | Gratuito | R$0 |
| Frontend (Vercel) | Hobby | R$0 |
| Firebase Auth | Spark (gratuito) | R$0 |
| Email (Resend) | 3k emails/mês | R$0 |
| **Total mês 1** | | **~R$35/mês** |

> 💡 Para volume maior de emails ou armazenamento de imagens, revisar limites do plano gratuito do MinIO (ou usar Cloudflare R2 gratuito como alternativa).

---

## 🔐 Segurança Mínima para MVP

Itens suficientes para operar sem expor o negócio a riscos óbvios:

- [x] Admin token (`X-Admin-Token`) protege rotas administrativas
- [ ] CORS apenas para origens autorizadas
- [ ] HTTPS obrigatório (redirect HTTP → HTTPS no Nginx)
- [ ] Firebase ID Token para rotas de cliente
- [ ] Variáveis sensíveis apenas em `.env` (nunca no git)
- [ ] `ADMIN_TOKEN` com valor forte (não `changeme`)
- [ ] Webhook secrets configurados corretamente (MP + Asaas)

> ❌ Para MVP em baixo volume, **podem ser adiados**: rate limiting, CSRF, WAF, audit trail completo.

---

## ✅ Checklist de Lançamento (Go Live)

### Backend
- [ ] CORS configurado com domínio real
- [ ] Firebase Auth middleware funcionando
- [ ] Estoque decrementado ao confirmar pagamento
- [ ] Webhook MP e Asaas testados em sandbox
- [ ] Health check respondendo `200 OK`
- [ ] `.env` configurado com credenciais reais (MP produção, Asaas produção)
- [ ] `ADMIN_TOKEN` forte e guardado com segurança

### Infraestrutura
- [ ] VPS provisionado com Docker instalado
- [ ] Domínio apontando para o IP da VPS
- [ ] Nginx configurado com reverse proxy
- [ ] SSL ativo (Let's Encrypt)
- [ ] `docker-compose up -d` rodando em produção
- [ ] Backup automático do MongoDB configurado (cron + `mongodump`)

### Frontend
- [ ] Variável `NEXT_PUBLIC_API_URL` apontando para a API em produção
- [ ] Login via Firebase funcionando
- [ ] Fluxo de compra completo testado (produto → carrinho → checkout → pagamento)
- [ ] Páginas com meta tags básicas (SEO)
- [ ] Deploy no Vercel (ou Netlify)

### Negócio
- [ ] Conta no Mercado Pago ou Asaas verificada (CPF/CNPJ do MEI)
- [ ] Pelo menos um produto cadastrado com estoque
- [ ] Configurações da loja preenchidas (`PUT /api/store`)
- [ ] Política de troca/devolução definida e exibida no site
- [ ] LGPD: aviso de cookies e política de privacidade básica

---

## 📚 Referências

- [Firebase Authentication (Backend Verify)](https://firebase.google.com/docs/auth/admin/verify-id-tokens)
- [Mercado Pago Webhooks](https://www.mercadopago.com.br/developers/pt/docs/your-integrations/notifications/webhooks)
- [Asaas API Docs](https://docs.asaas.com/)
- [Let's Encrypt via Certbot](https://certbot.eff.org/)
- [Resend Email API](https://resend.com/docs)
- [shadcn/ui](https://ui.shadcn.com/) — componentes React prontos para o frontend

---

*Última atualização: Maio 2026*
