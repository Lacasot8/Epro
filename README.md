# Mpro v2.0 - com suporte xHTTP (SplitHTTP)

## Núcleo HCR independente em validação

O repositório contém `src/hcr.rs`, um núcleo Rust independente para validar o enquadramento binário observado no cliente HCR. O cabeçalho de request tem 62 bytes: opcode, versão 1, SessionID de 16 bytes, token de 32 bytes, sequência uint64 big-endian nos bytes 50–57 e comprimento big-endian nos bytes 58–61. O limite de payload observado é 262144 bytes. O módulo ainda não implementa autenticação, ciclo de sessão, upload/download nem substitui o servidor HCR usado em produção; não baixa, incorpora nem executa binários de terceiros.

Em runtime, a opção H nas portas 80/8080 reconhece o frame e encaminha o fluxo TCP intacto para `127.0.0.1:8880`; ela não cria por si só um servidor HCR independente. Uma conexão direta do APK para a porta 8880 não percorre esse relay do AWpro. O menu ainda depende de um backend HCR instalado separadamente.

Esse núcleo ainda não substitui o servidor HCR usado em produção. A análise estática do APK identificou o campo de token e o layout do frame, mas não comprovou a autenticação nem a semântica de todas as operações e da reconexão. Sem esses detalhes — idealmente confirmados por uma captura autorizada de sessão cliente-servidor — ativar um servidor incompleto poderia causar EOF em clientes como YouTube e Speedtest. Por isso, o caminho atual do menu permanece preservado até o ciclo de sessão, upload, download e reconexão serem comprovados.

Proxy multi-protocolo em Rust com suporte exclusivo para o protocolo xHTTP (SplitHTTP) na porta 443, compatível com o cliente **XHTTP-DEMO**.

## O que mudou na v2.0

| Recurso | v1.0 | v2.0 |
|---------|------|------|
| xHTTP/SplitHTTP exclusivo | Não | Sim (opção 04) |
| TLS termination | Passthrough | Terminação com rustls |
| Porta 443 automática | Manual | Automática (opção 04) |
| Certificados auto-assinados | Não | Gerados automaticamente |
| UDP + QUIC na 443 | Manual | Auto-ativados |
| SocksRevive compatibilidade | Não | Sim |

## Instalação

```bash
# Clonar ou copiar os arquivos para o servidor
git clone https://github.com/Lacasot8/Epro.git
cd Epro

# Substituir os arquivos pelo v2.0
cp menu.sh menu_original.sh.backup
# Copiar os novos arquivos para o repositório

# Compilar e instalar
chmod +x install.sh
./install.sh
```

## Uso

```bash
epro    # Comando principal para abrir o menu deste proxy
```

### Opção 01 — Configurar portas

**[01] CONFIGURAR PORTAS** abre um submenu com **[01] ABRIR PORTA**, **[02] HCR ABRIR PORTA** e **[03] HCR INSTALAR / ATUALIZAR SERVIDOR**. A opção normal mantém o fluxo HTTP legado compatível com a opção 1 do AWProxy e pergunta se HTTPS/TLS deve ser ativado. A opção HCR é separada e usa o binário oficial `hcr-server` para Linux amd64. Se o binário não estiver presente em `/opt/mpro/hcr-server`, `/opt/hcr-server/hcr-server`, `/usr/local/bin/hcr-server` ou `/usr/bin/hcr-server`, o menu oferece baixar e instalar a versão oficial automaticamente (valida ELF64 amd64, guarda backup do binário anterior e pode usar outra origem via `HCR_SOURCE_URL`); recusando a instalação, o menu apenas mostra onde colocar o binário. Em seguida ele cria e inicia `hcr-8880.service` com transporte `plain` (sem TLS) e destino SSH `127.0.0.1:22`, detecta o formato de argumentos aceito pelo binário (`-listen/-target/-transport` ou `--port/--transport`) e tenta o formato alternativo se a porta não subir, verifica conflitos de porta, confirma que a 8880 está escutando e faz rollback da unit se o serviço falhar. Uma unit `hcr-8880.service` já existente nunca é sobrescrita: o menu só pergunta se deve iniciá-la. O handler HCR TLS legado do modo multiprotocolo não substitui este serviço HCR plain.

Na opção 1, o handler segue o fluxo do WZPRO: envia `HTTP/1.1 101 Switching Protocols`, lê até 1024 bytes, envia `HTTP/1.1 200 <status>` e então seleciona o backend. Não envia o segundo `101` nem retransmite aquele buffer inicial ao backend. A sequência `101`, `101`, `200` com o mesmo texto e o encaminhamento do preâmbulo continuam reservados ao multiprotocolo quando ele está ativo para a porta. O texto `<status>` vem da configuração do serviço, por exemplo `Epro`. Se “somente SSH” estiver ativa, encaminha ao SSH local (`127.0.0.1:22`); caso contrário, a seleção inspeciona os próximos bytes com `peek` por até 200 ms e escolhe SSH ou VPN local (`127.0.0.1:1194`). O padrão do Epro é `@Epro`.

No menu, escolha **[06] ATUALIZAR PELO GITHUB** para baixar a versão mais recente do branch `main`, recompilar os binários e atualizar o menu. Os serviços Epro que estavam ativos são parados durante a troca e iniciados novamente ao final; as configurações systemd e portas desligadas são preservadas. O instalador reaproveita o cache de compilação em `/var/cache/epro/cargo-target`, instala dependências somente quando faltam e usa paralelismo limitado pela memória disponível. A primeira compilação ainda pode demorar mais; o tempo varia conforme VPS, rede e cache.

### Menu v2.0

```
╔══════════════════════════════════╗
║       Mpro Menu Free v2.0     ║
╠══════════════════════════════════╣
║                                  ║
║ [01] - CONFIGURAR PORTAS         ║
║ [02] - FECHAR PORTA              ║
║ [03] - REINICIAR PORTA           ║
║ [04] - xHTTP SPLITHTTP (443)    ║
║                                  ║
║ [00] - SAIR                      ║
╚══════════════════════════════════╝
```

O submenu de configuração de portas contém:

```
[01] - ABRIR PORTA
[02] - HCR ABRIR PORTA (8880 / PLAIN)
[03] - HCR INSTALAR / ATUALIZAR SERVIDOR
```

### Opção 04 - xHTTP SplitHTTP (Porta 443)

Esta opção é **exclusiva** para o protocolo xHTTP (SplitHTTP) e funciona assim:

1. **Porta fixa 443** - Não pode ser alterada
2. **TLS obrigatório** - Certificado auto-assinado gerado automaticamente
3. **SSH only** - Tunnel para SSH local (127.0.0.1:22)
4. **UDP + QUIC** - Ativados automaticamente
5. **Status HTTP** - Cabeçalho retornado ao cliente (padrão: @Mpro)

### Fluxo do xHTTP

```
Cliente (SocksRevive)          Servidor (Mpro)
       │                              │
       ├── TCP connect :443 ──────────┤
       ├── TLS ClientHello ───────────┤
       │                              ├── TLS handshake (rustls)
       │                              ├── TLS ServerHello
       │                              ├── TLS decodificado
       │                              │
       ├── HTTP/2 GET /ssh/{id} ─────┤
       │                              ├── Sessão criada
       │                              ├── Conecta SSH backend
       │                              ├── Streaming chunked
       │                              │
       │◄── HTTP 200 + chunks ────────┤ (dados SSH do servidor)
       │                              │
       ├── HTTP/2 POST /ssh/{id}/0 ──┤ (dados SSH do cliente)
       │                              ├── Encaminha para SSH backend
       │                              │
       ├── HTTP/2 POST /ssh/{id}/1 ──┤
       │                              ├── Encaminha para SSH backend
```

### Configuração no SocksRevive-XHTTP-DEMO

| Campo | Valor |
|-------|-------|
| Server | IP do servidor Mpro |
| Port | 443 |
| SNI | Qualquer domínio (trust-all) |
| XHTTP Host | IP do servidor ou vazio |
| XHTTP Path | /ssh |
| XHTTP TLS | HABILITADO |
| Username | Usuário SSH |
| Password | Senha SSH |

## Flags do Binário

```
-p PORTA       Porta TCP (padrão: 80)
-s STATUS      Status HTTP (padrão: @Mpro)
-t             Habilitar TLS
-ssh           SSH only (tunnel para 127.0.0.1:22)
-u             Habilitar UDP
-q             Habilitar QUIC
-x             xHTTP mode (força porta 443, TLS, SSH only)
```

## Notas sobre TLS

O TLS na v2.0 usa **terminação local** com `rustls`:

1. O certificado e chave são gerados em `/opt/mpro/`
2. O proxy faz handshake TLS completo com o cliente
3. Os dados decodificados são roteados para o handler correto
4. Para xHTTP: os dados HTTP/2 ficam visíveis após TLS

## Troubleshooting

```bash
# Ver logs
journalctl -u proxy-443.service -f

# Verificar SSH
systemctl status ssh

# Testar TLS
openssl s_client -connect 127.0.0.1:443

# Verificar porta
ss -tlnp | grep 443
```

## Requisitos

- Rust 1.70+
- systemd
- OpenSSH Server (rodando na porta 22)
- OpenSSL (para certificados)
- Porta 443 aberta no firewall

## Opção 07 — Multiprotocolo + Status HTTP + TCP

A opção **[07] MULTIPROTOCOLO + STATUS + TCP** lista as portas numéricas Epro ativas e permite ativar/desativar o dispatcher de protocolos em cada porta. A tecla **H** abre **ATIVAR SUPORTE AO HCR**, com estado ON/OFF independente em cada porta. Ativar HCR não cria, remove nem altera o marcador `multiprotocol-PORT.enabled`; quando o multiprotocolo está OFF, frames HCR são encaminhados ao servidor HCR oficial e o restante continua no protocolo da opção 1. Quando os dois estão ON, frames HCR válidos são tratados antes do dispatcher e os demais protocolos continuam no dispatcher. A detecção aguarda até 8 segundos por um cabeçalho HCR completo de 62 bytes, aceitando frames fragmentados em redes móveis. O serviço HCR oficial da porta 8880 precisa estar ativo. Nenhuma dessas opções abre ou fecha portas.

**HCR nas portas 80/8080 (APK AZPro Lite).** Nas redes móveis, as portas 80/8080 passam por proxy transparente da operadora, que derruba tráfego binário cru, e conexões novas direto na 8880 podem ser descartadas no caminho (`failed to connect ... after 4000ms` no log do app, sem nenhum descarte registrado na VPS). Por isso, com HCR ON na porta (tecla H), o AWpro também aceita HCR encapsulado em HTTP: o cliente abre cada conexão com `GET /hcr HTTP/1.1` + `Upgrade: websocket` (ou cabeçalho `X-AzPro-HCR`), o AWpro responde `101 Switching Protocols` com `Sec-WebSocket-Accept` e repassa os frames HCR seguintes intactos ao `hcr-server` em `127.0.0.1:8880`. Pedidos HTTP sem essa marca continuam no fluxo normal da porta, e o HCR cru continua funcionando. O APK AZPro Lite precisa do patch em `apk-patch/` (classe `HcrHttpUpgrade`, chamada pelo `HcrSocketRelay` logo após o `connect`), que faz esse handshake em qualquer porta exceto a 8880. Cada alteração reinicia o serviço da porta e derruba as conexões existentes. As opções 01–06 permanecem separadas.

Com o modo ligado, o serviço detecta SOCKS4/SOCKS4a, quadros HCR, conexões TLS, WebSocket RFC 6455, SECURITY, HTTP e SSH direto. Cabeçalhos HCR válidos de 62 bytes (opcodes 1–6) são reconhecidos antes do roteamento comum — inclusive quando os bytes iniciais se sobrepõem ao SOCKS4/5 — e encaminhados intactos ao servidor HCR oficial em `127.0.0.1:8880` quando HCR está ON naquela porta. Quadros HCR parciais não são consumidos; handshakes TLS e preâmbulos não reconhecidos também seguem ao servidor HCR quando ON. Com HCR OFF, TLS segue o passthrough legado e tráfego desconhecido usa o fallback SSH/VPN. No HTTP/SSH_PROXY, o dispatcher envia a sequência multi-status esperada pelo cliente (`101 <status>`, `101 <status>`, `200 <status>`), remove os blocos HTTP camuflados do preâmbulo — inclusive payloads encadeadas com `GET`, `GET-` e `UNLOCK` — e mantém o relay bidirecional. Bytes SSH reais que cheguem depois dos decoys são preservados e encaminhados ao `sshd`; os headers HTTP não são encaminhados. O fluxo legado de ABRIR PORTA permanece separado. O texto de status é o configurado na flag `-s`. WebSocket usa os códigos/reason phrases exigidos pelo protocolo. SOCKS4/5 só permitem CONNECT à porta 22; o cliente não pode escolher host ou porta arbitrários.

O relay HCR usa buffers bidirecionais de 128 KiB, janela de recebimento TCP de 512 KiB e envio de 256 KiB. O envio é limitado por `TCP_NOTSENT_LOWAT` em 64 KiB para vencer RTT alto sem acumular uma fila enorme e elevar o ping. Quando o kernel oferece BBR, ele é aplicado por conexão; caso contrário, o congestion control existente é preservado. O servidor HCR é iniciado com até 4096 conexões, 1024 sessões e 1024 sessões por origem. O limite por origem é alto de propósito: pela opção H nas portas 80/8080 todos os clientes chegam ao `hcr-server` como `127.0.0.1`, e o limite antigo de 64 bloqueava usuários. Ao abrir o menu depois de atualizar pelo GitHub, esses valores e o tamanho de frame são reaplicados automaticamente na unit `hcr-8880.service` existente, que é reiniciada uma única vez se algo mudou. Cada porta com HCR ON recebe uma dependência systemd persistente de `hcr-8880.service`, e o proxy tenta reconectar ao backend por até 20 segundos após reinícios. O menu configura `-download-poll-timeout 30s`, valor compatível com o hcr-server 0.1.29, e mantém keepalive TCP/SSH para reduzir resets durante o polling direto na porta 8880. Os frames de download HCR usam 64 KiB (o cliente AZPro Lite aceita até 256 KiB), reduzindo o número de polls em RTT alto.

O estado multiprotocolo é individual por porta, no marcador `/opt/mpro/multiprotocol-<porta>.enabled`; o estado HCR fica em `/opt/mpro/hcr-<porta>.enabled`. Ativar uma porta que não tenha OpenSSH na 22 e, quando aplicável, VPN/OpenVPN na 1194 fará o respectivo protocolo falhar; verifique os serviços locais antes de testar. O listener UDP existente apenas registra datagramas e QUIC continua sem listener funcional; a opção 7 ativa despacho TCP, não transforma esses componentes em serviços UDP/QUIC.

## Opção 08 — Desinstalar Proxy

A opção **[08] DESINSTALAR PROXY** exige execução como root. O menu explica: digite `s` para desinstalar ou `n` para cancelar; somente `s` ou `S` confirma. Ela para e desabilita somente os serviços systemd cuja linha `ExecStart` aponta para os binários Epro em `/opt/mpro`, remove os links instalados em `/usr/local/bin` quando ainda apontam para o Epro e apaga os binários, menu, certificados TLS e marcadores BYPASS/legados conhecidos. Não remove OpenSSH, Rust, OpenSSL, regras de firewall nem arquivos desconhecidos que estejam em `/opt/mpro`; se sobrarem arquivos, o diretório é preservado.
