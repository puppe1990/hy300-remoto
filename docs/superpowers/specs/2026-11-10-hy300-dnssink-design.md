# HY300 DNS sinkhole (pyhole) — dnsmasq de fábrica

Data: 2026-11-10  
Pasta: `produtos/hy300-remoto`  
Status: approved (abordagem 1)  
Aparelho: HY300 `192.168.15.4:3268` (ADB root, SELinux Permissive)

DNS sinkhole para a LAN Puppe usando o `dnsmasq` 2.51 que já está em `/system/bin/dnsmasq`. Sem Pi-hole, sem Python, sem Termux, sem painel web.

## Objetivo

A casa consulta `192.168.15.4:53`. Domínios da lista StevenBlack (adware + malware) respondem `0.0.0.0`. O resto vai para `1.1.1.1` e, se esse falhar, `8.8.8.8`. DNS 2 em cada cliente é `1.1.1.1`: projetor desligado → anúncio volta, internet não cai.

## Fora deste ciclo

- Pi-hole oficial, `pyhole/pyhole`, AdGuard Home, blocky, Termux, proot
- Painel web, estatística, query log permanente
- DHCP (o dnsmasq 2.51 tem DHCP compilado; **nunca** passar `-F` / `dhcp-range`)
- Hotspot / tether do HY300 (SoftAP continua off)
- IPv6 DNS (clientes que usam o DNS da RA da Vivo furam o buraco; v1 é IPv4)
- Mexer em `daemon12138`, TXCZ, OpenFlix, Magisk
- Remontar `super.img` / gravar na partição de sistema de verdade (só overlay `adb remount` no scratch)
- Alterar o DHCP da Vivo (o menu costuma não deixar; cada aparelho aponta o DNS na mão)

## Por que este recorte

O HY300 é Android 12 `armeabi-v7a`, ~2 GB RAM (~370 MB livres), `/data` com 3 GB livres, porta 53 livre, `dnsmasq` 2.51 (2009, IPv6 + DHCP, sem scripts). `busybox wget` é de 2014 e o TLS quebra em GitHub. O Mac busca a lista e empurra por ADB.

## Arquitetura

```
cliente LAN  --UDP/TCP 53-->  dnsmasq no HY300 (wlan0 + 127.0.0.1)
                                  | hit na lista  -->  0.0.0.0
                                  | miss          -->  1.1.1.1, depois 8.8.8.8
```

Três peças:

1. **Payload em `/data/local/dnssink/`** (sobrevive reboot; `/data` é userdata).
2. **Serviço init** `/system/etc/init/dnssink.rc` no overlay de `adb remount` (scratch f2fs ~396 MB, já usado neste userdebug).
3. **Scripts no Mac** `scripts/dnssink/` neste repo: instalar, atualizar lista, desinstalar, apontar o DNS do Mac.

O binário continua `/system/bin/dnsmasq`. Não copiamos outro.

## Payload no aparelho

| Caminho                             | Função                                                                                |
| ----------------------------------- | ------------------------------------------------------------------------------------- |
| `/data/local/dnssink/start.sh`      | Espera `wlan0` ter IPv4 (até 60 s), grava o conf, pega `wake_lock`, `exec dnsmasq -k` |
| `/data/local/dnssink/dnsmasq.conf`  | Gerado pelo `start.sh` (IP atual no `-a`)                                             |
| `/data/local/dnssink/hosts`         | StevenBlack, formato `0.0.0.0 dominio`                                                |
| `/data/local/dnssink/whitelist.txt` | Um domínio por linha; o update no Mac tira esses nomes da lista                       |
| `/data/local/dnssink/dnsmasq.pid`   | PID                                                                                   |
| `/data/local/dnssink/dnsmasq.log`   | Só se debug; v1 produção sem `--log-queries`                                          |

`start.sh` é o único processo que o init vê: espera a interface, escreve o conf, `echo dnssink > /sys/power/wake_lock`, `exec /system/bin/dnsmasq -C ... -k -x ...`. Init reinicia se cair.

Conf gerado (flags do 2.51; `-a` uma vez por endereço):

```
domain-needed
bogus-priv
no-resolv
server=1.1.1.1
server=8.8.8.8
strict-order
no-hosts
addn-hosts=/data/local/dnssink/hosts
listen-address=127.0.0.1
listen-address=<ipv4-wlan0>
bind-interfaces
interface=wlan0
no-dhcp-interface=wlan0
cache-size=10000
pid-file=/data/local/dnssink/dnsmasq.pid
user=root
```

Nunca `dhcp-range`. Nunca `--resolv-file` apontando para o DNS da Vivo (loop se alguém puser o projetor como DNS do próprio projetor). Upstream fixo `1.1.1.1` / `8.8.8.8`.

O HY300 **não** muda o próprio `net.dns*`: ele continua no DHCP da Vivo (`192.168.15.1`). Só a LAN usa `192.168.15.4`. Assim o daemon não consulta a si mesmo.

## Boot

`/system/etc/init/dnssink.rc` (overlay, não a partição dm):

```
service dnssink /system/bin/sh /data/local/dnssink/start.sh
    class late_start
    user root
    group root inet
    seclabel u:r:su:s0
    disabled

on property:sys.boot_completed=1
    start dnssink
```

Mesmo gancho que `vendor.daemon12138.rc` (`sys.boot_completed=1`). Não editar o rc do daemon da lâmpada.

`adb remount` neste userdebug já sobe overlay rw em `/system` via `/mnt/scratch`. A mensagem de reboot do `remount` é genérica; gravar o `.rc` e reboot **uma vez** no install para o init reler. Sem Magisk.

## Lista

Fonte: `https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts` (adware + malware; sem fakenews/gambling/porn).

O Mac baixa, tira comentário e `localhost`, aplica `whitelist.txt`, recusa se o arquivo passar de **20 MB** (RAM do RK3326). Empurra para `/data/local/dnssink/hosts` e manda `SIGHUP` no PID (dnsmasq 2.51 relê `addn-hosts` no HUP).

`busybox wget` no aparelho fica de fora (BusyBox 1.22, TLS velho). Update = script no Mac, sob demanda. Sem cron.

YouTube/OpenFlix no projetor: DNS não come anúncio in-stream (mesmo host do vídeo). Come tracker, malvertising e boa parte de banner em app/site.

## Clientes

DNS 1 = `192.168.15.4`, DNS 2 = `1.1.1.1`.

- Mac (`Wi-Fi`): `networksetup -setdnsservers Wi-Fi 192.168.15.4 1.1.1.1`
- iPhone / Android / outros: DNS manual na Wi-Fi Puppe
- Router Vivo: não é requisito; se um dia o menu deixar, o DHCP pode repetir o mesmo par

Standby:

- Screensaver / tela apagada: Wi-Fi `wifi_sleep_policy=2` (nunca dorme), aparelho na tomada (`mStayOn=true`). Com `wake_lock` `dnssink`, o daemon segue.
- Standby da lâmpada (power curto, `daemon12138`): SoC em geral fica; DNS segue.
- Desligar de verdade / tomada: a casa cai no DNS 2 (`1.1.1.1`), sem bloqueio, com internet.

## Wakelock e energia

Nome `dnssink` em `/sys/power/wake_lock` no start; `/sys/power/wake_unlock` no uninstall. O arquivo existe neste kernel. Não usar Doze whitelist de app (não há APK).

## Scripts no Mac

`produtos/hy300-remoto/scripts/dnssink/`, ADB `192.168.15.4:3268` (o mesmo host do app).

| Script                 | Faz                                                                                                                                    |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `install.sh`           | `adb root`, pasta em `/data`, push start.sh + hosts + whitelist, `adb remount`, push `dnssink.rc`, `start dnssink` (ou reboot uma vez) |
| `update-hosts.sh`      | curl StevenBlack, filtra, cap 20 MB, push, HUP                                                                                         |
| `uninstall.sh`         | stop, wake_unlock, rm payload, rm rc do overlay, DNS do Mac volta a DHCP (`networksetup -setdnsservers Wi-Fi Empty`)                   |
| `configure-mac-dns.sh` | aponta o Mac para `192.168.15.4` `1.1.1.1`                                                                                             |

O builder da lista (filtro + whitelist + cap) é função testável em JS (`scripts/dnssink/build-hosts.mjs`) com `@faker-js/faker`: domínio falso na lista some; domínio na whitelist permanece resolvível (não entra no hosts de saída); arquivo > 20 MB lança erro com o tamanho recebido e o teto.

Testes **não** falam com o projetor (regra do `AGENTS.md`). Smoke no aparelho é manual, depois do install.

## Erros

- `wlan0` sem IPv4 em 60 s: `start.sh` sai ≠ 0; init tenta de novo. Mensagem inclui a interface e o timeout.
- `dnsmasq` não sobe: log `logcat -s dnsmasq` / stderr do service; causa comum é porta 53 ocupada (tether). Erro cita o bind e o IP tentado.
- Lista > 20 MB: update aborta, hosts antigo permanece.
- IP do projetor muda: bind-interfaces fica surdo no endereço velho. Neste LAN o lease tem sido `192.168.15.4`; se mudar, rodar `start.sh` de novo (reboot também).

## Verificação (manual, no Mac, depois do install)

```
dig @192.168.15.4 doubleclick.net +short    # 0.0.0.0
dig @192.168.15.4 example.com +short        # IPv4 real
dig @192.168.15.4 google.com +short         # IPv4 real
```

No Mac, com DNS já apontado: o mesmo `doubleclick.net` cai em `0.0.0.0` sem `@`. `scutil --dns` mostra `192.168.15.4` e `1.1.1.1`.

## Rollback

`uninstall.sh`. Sem o rc e sem o processo, a LAN volta a usar o DNS 2 / DHCP. Overlay scratch só perde o arquivo `dnssink.rc`; o resto do sistema fica.

## Riscos

- Projetor off sem DNS 2 nos clientes = casa sem internet de nome.
- `dnsmasq` 2.51 + lista grande: teto 20 MB para não comer a RAM do RK3326.
- Overlay `adb remount` some em factory reset; reinstall.
- IPv6 DNS da operadora fura o bloqueio até o cliente forçar IPv4 DNS.
