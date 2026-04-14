# 🎯 Setup Simplificado - OmniStore para MEIs

> **Para quem?** Donos de pequenas lojas que querem sua própria API  
> **Nível técnico**: Iniciante (vamos te guiar passo a passo)  
> **Tempo**: 1 hora (primeira vez)

---

## 🤔 O que é isso?

OmniStore é uma **API headless** - pense nela como o "cérebro" da sua loja online. Ela:

- ✅ Guarda seus produtos, pedidos, clientes
- ✅ Processa pagamentos (Mercado Pago, Asaas)
- ✅ Gerencia estoque
- ✅ Pode ser usada com **qualquer** site/app (WordPress, Next.js, mobile app, etc)

**Vantagem**: Você não fica preso a uma plataforma. Mude o visual da loja sem perder os dados!

---

## 📋 Checklist: O que você precisa ter

Antes de começar, prepare:

- [ ] **Computador** com internet (Windows, Mac ou Linux)
- [ ] **Cartão de crédito** para contratar VPS (~R$ 25/mês)
- [ ] **30 minutos** de tempo livre
- [ ] **Paciência** (é mais simples do que parece!)

**Opcional** (pode fazer depois):
- [ ] Domínio próprio (api.minhaloja.com.br)
- [ ] Conta Mercado Pago ou Asaas

---

## 🏁 Passo 1: Contratar um Servidor (VPS)

### O que é VPS?
É um "computador na nuvem" que fica ligado 24/7 rodando sua loja.

### Onde contratar? (escolha um)

#### Opção A: Hetzner (MAIS BARATO - €4/mês ≈ R$ 23)

1. Acesse: https://www.hetzner.com/cloud
2. Clique em "Sign Up" (criar conta)
3. Preencha email e senha
4. Confirme email
5. Escolha:
   - **Location**: Germany (ou Finland)
   - **Image**: Ubuntu 22.04
   - **Type**: CX11 (2GB RAM)
   - **Networking**: IPv4
6. Clique em "Create & Buy Now"
7. **Anote o IP que aparecer** (ex: 123.45.67.89)
8. **Anote a senha** que vai aparecer na tela (ou chegará no email)

#### Opção B: DigitalOcean (MAIS FÁCIL - $6/mês ≈ R$ 30)

1. Acesse: https://www.digitalocean.com
2. Clique em "Sign Up"
3. Use GitHub ou Google para login rápido
4. Clique em "Create Droplet"
5. Escolha:
   - **Image**: Ubuntu 22.04 LTS
   - **Plan**: Basic (Regular, $6/mo, 1GB)
   - **Region**: São Paulo (Brazil) ou New York
6. Crie senha SSH ou use sua chave (se não entender, deixe senha)
7. Clique em "Create Droplet"
8. **Anote o IP** que aparecer

💡 **Dica**: Copie IP e senha para um bloco de notas!

---

## 💻 Passo 2: Conectar no Servidor

### No Windows

1. Baixe **PuTTY**: https://www.putty.org/
2. Abra PuTTY
3. Em "Host Name", cole o IP do seu servidor
4. Clique em "Open"
5. Na tela preta que abrir:
   - Login: `root`
   - Password: (cole a senha - não vai aparecer nada, é normal!)

### No Mac/Linux

1. Abra o **Terminal** (procure por "Terminal" no Spotlight/Dash)
2. Digite:
   ```bash
   ssh root@SEU-IP-AQUI
   ```
   (substitua SEU-IP-AQUI pelo IP real)
3. Digite "yes" quando perguntar
4. Cole a senha (não vai aparecer, é normal!)

✅ **Se aparecer um texto de boas-vindas do Ubuntu, deu certo!**

---

## 🔧 Passo 3: Instalar o Necessário

Agora você está "dentro" do servidor. Copie e cole estes comandos:

### 3.1 Atualizar sistema

```bash
apt update && apt upgrade -y
```

⏳ Aguarde 2-5 minutos...

### 3.2 Instalar Docker

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
```

⏳ Aguarde 1-2 minutos...

### 3.3 Instalar Docker Compose

```bash
apt install docker-compose -y
```

### 3.4 Verificar

```bash
docker --version
```

Se aparecer algo como `Docker version 24.x.x`, está instalado! ✅

---

## 📥 Passo 4: Baixar OmniStore

```bash
cd /root
git clone https://github.com/seu-usuario/omnistore.git
cd omnistore
```

💡 Se pedir usuário/senha do GitHub:
- Usuário: seu GitHub username
- Senha: use Personal Access Token (https://github.com/settings/tokens)

---

## ⚙️ Passo 5: Configurar Variáveis

### 5.1 Copiar arquivo de exemplo

```bash
cp .env.example .env
```

### 5.2 Editar configurações

```bash
nano .env
```

Você verá um editor de texto. Use as **setas** do teclado para navegar.

### 5.3 O que PRECISA mudar:

Encontre estas linhas e mude:

```bash
# MUDE ESTA SENHA (MongoDB)
MONGO_URI=mongodb://admin:SENHA_FORTE_AQUI@mongo:27017

# MUDE ESTA SENHA (MinIO - para uploads)
S3_SECRET_KEY=OUTRA_SENHA_FORTE_AQUI

# MUDE ESTE TOKEN (admin da API)
ADMIN_TOKEN=CRIE_UMA_SENHA_COMPLICADA_AQUI
```

💡 **Dica para criar senha forte**:
```bash
# Abra outro terminal e digite:
openssl rand -base64 32
# Copie o resultado
```

### 5.4 Opcional: Adicionar Mercado Pago

Se já tem conta:
```bash
MERCADO_PAGO_ACCESS_TOKEN=seu_token_do_mercadopago
```

Pegar em: https://www.mercadopago.com.br/developers/panel/app

### 5.5 Salvar e sair do editor

- Aperte `Ctrl + X`
- Aperte `Y` (yes)
- Aperte `Enter`

✅ Arquivo salvo!

---

## 🚀 Passo 6: Iniciar a Loja

```bash
docker-compose up -d
```

⏳ Aguarde 3-5 minutos (está baixando e instalando tudo)...

### Verificar se subiu

```bash
docker-compose ps
```

Deve mostrar 4 containers rodando:
- ✅ omnistore-app
- ✅ omnistore-mongo
- ✅ omnistore-redis
- ✅ omnistore-minio

### Testar se funciona

```bash
curl http://localhost:8080/health
```

Se aparecer `{"status":"ok"}`, **PARABÉNS! SUA API ESTÁ RODANDO!** 🎉

---

## 🌐 Passo 7: Deixar Acessível pela Internet

Atualmente só você consegue acessar. Vamos abrir para internet:

### 7.1 Instalar Nginx

```bash
apt install nginx -y
```

### 7.2 Criar configuração

```bash
nano /etc/nginx/sites-available/omnistore
```

Cole isso (substitua `SEU-IP` pelo IP do servidor):

```nginx
server {
    listen 80;
    server_name SEU-IP;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

Salve: `Ctrl+X`, `Y`, `Enter`

### 7.3 Ativar

```bash
ln -s /etc/nginx/sites-available/omnistore /etc/nginx/sites-enabled/
nginx -t
systemctl restart nginx
```

### 7.4 Abrir firewall

```bash
ufw allow 80
ufw allow 443
ufw allow 22
ufw enable
```

Digite `y` quando perguntar.

---

## ✅ Passo 8: Testar do Seu Computador

No **seu computador** (não no servidor), abra navegador e acesse:

```
http://SEU-IP/admin/stats
```

Se aparecer um JSON com estatísticas, **FUNCIONOU!** 🎉

---

## 📱 Como Usar Agora?

### Opção 1: Testar com Postman/Insomnia

1. Baixe **Postman**: https://www.postman.com/downloads/
2. Importe esta coleção: [link para collection]
3. Mude `BASE_URL` para `http://SEU-IP`
4. Teste criar produtos, pedidos, etc

### Opção 2: Criar um Site

Sua API está pronta! Agora você pode:

- 🌐 Criar site em **Next.js** que consome a API
- 🛒 Usar **WordPress + WooCommerce** conectado na API
- 📱 Criar **app mobile** React Native
- 💻 Qualquer coisa que faça requisições HTTP!

Exemplos de código em: [docs/examples/](docs/examples/)

---

## 🆘 Algo Deu Errado?

### "Connection refused" ao acessar

```bash
# Verificar se está rodando
docker-compose ps

# Ver logs de erro
docker-compose logs app
```

### "Out of memory"

Seu servidor tem pouca RAM. Adicione swap:

```bash
fallocate -l 1G /swapfile
chmod 600 /swapfile
mkswap /swapfile
swapon /swapfile
```

### Esqueci a senha do servidor

- **Hetzner**: Console no painel > Reset root password
- **DigitalOcean**: Acesse droplet > Access > Reset Root Password

### Docker dá erro

```bash
# Reiniciar Docker
systemctl restart docker

# Reconstruir containers
cd /root/omnistore
docker-compose down
docker-compose up -d --build
```

---

## 🎓 Próximos Passos

Agora que está rodando:

1. **Configure um domínio** (api.minhaloja.com.br)
   - Veja: [DEPLOYMENT_GUIDE.md - Seção DNS](DEPLOYMENT_GUIDE.md#-configuração-de-dns)

2. **Adicione HTTPS** (cadeado verde)
   ```bash
   apt install certbot python3-certbot-nginx -y
   certbot --nginx -d api.minhaloja.com.br
   ```

3. **Configure backup automático**
   - Veja: [DEPLOYMENT_GUIDE.md - Seção Backup](DEPLOYMENT_GUIDE.md#-backup-automático)

4. **Monte seu frontend**
   - Template Next.js: [github.com/omnistore/nextjs-template]
   - Template WordPress: [github.com/omnistore/wp-plugin]

---

## 💡 Dicas para MEIs

### Quanto custa manter?

- VPS: R$ 25/mês (Hetzner) ou R$ 30/mês (DigitalOcean)
- Domínio: R$ 40/ano (registro.br)
- SSL: Grátis (Let's Encrypt)
- **Total**: ~R$ 28/mês

Compare com:
- Shopify: R$ 130/mês
- Nuvemshop: R$ 90/mês
- VTEX: R$ 500+/mês

### Vale a pena?

✅ **SIM, se você**:
- Quer controle total dos dados
- Planeja escalar sem custos crescentes
- Quer integrar com outros sistemas
- Tem um desenvolvedor ou está aprendendo

❌ **TALVEZ NÃO, se você**:
- Nunca usou computador além de redes sociais
- Não tem tempo para aprender básico de tecnologia
- Precisa de suporte 24/7 urgente

---

## 🎉 Parabéns!

Você acabou de colocar sua própria API de e-commerce no ar!

Isso é **muito** mais do que 90% das lojas fazem. Agora você tem:
- ✅ Controle total dos seus dados
- ✅ Sem taxas por transação (do sistema, MP/Asaas cobram normal)
- ✅ Escalabilidade ilimitada
- ✅ Integração com qualquer frontend

**Próximo desafio**: Criar o site que vai usar essa API! 🚀

---

## 📚 Materiais de Apoio

- [Vídeo: Como funciona uma API?](https://youtube.com/watch?v=exemplo)
- [Curso gratuito: Postman Básico](https://youtube.com/channel/exemplo)
- [Documentação da API](API_REFERENCE.md)
- [Comunidade Discord](https://discord.gg/exemplo)

---

**Dúvidas?** 
- 📧 Email: suporte@omnistore.dev
- 💬 Discord: [Link]
- 🐛 Achou um erro? Abra issue no GitHub

---

Feito com ❤️ para MEIs brasileiros 🇧🇷
