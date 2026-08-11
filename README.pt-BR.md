# easy1090

[English](README.md) · **Português**

Instalador do stack ADS-B completo (RTL-SDR + readsb + tar1090) em um comando, para Arch e derivados.

Guias de ADS-B quase sempre assumem Raspberry Pi OS ou Debian: os scripts oficiais usam `apt-get`, o `lighttpd` do Debian já vem com a estrutura `conf.d`/`conf-enabled` pronta e os pacotes AUR de decodificadores antigos não são testados contra o GCC atual. Cada um desses pontos custa depuração real numa distro Arch-based. O easy1090 é a automação disso, com os atritos já resolvidos.

No fim você tem: driver correto para a RTL-SDR Blog V4, `readsb` decodificando ADS-B em 1090 MHz e um mapa web ao vivo, tudo com permissões, regras `udev` e serviços `systemd` no lugar, sobrevivendo a reboot.

## Requisitos

- Arch Linux ou derivado (EndeavourOS, Manjaro, CachyOS)
- Um helper de AUR (`yay`)
- Uma RTL-SDR conectada, de preferência a RTL-SDR Blog V4
- Um terminal de verdade, porque o `sudo` precisa de tty

## Uso

```bash
git clone https://github.com/Esl1h/easy1090.git
cd easy1090
./install.sh
```

Antes de rodar qualquer coisa como root na sua máquina, veja o que ele faria:

```bash
./install.sh --dry-run
```

O `--dry-run` roda o preflight inteiro (que é só leitura) e imprime os comandos exatos, não descrições deles. É o mesmo texto que você poderia copiar e colar no terminal.

Na primeira execução ele pergunta o idioma da interface (português ou inglês), as coordenadas da sua antena e se você quer compartilhar dados com uma rede pública de rastreamento de voos. Os três ficam gravados no `install.conf`.

### Opções

```
--full              instala tudo, inclusive SDR++ e SatDump
--lang <pt|en>      idioma da interface
--lat <graus>       latitude da antena (ex: -23.58)
--lon <graus>       longitude da antena (ex: -46.55)
--skip-tar1090      não instala o mapa web
--skip-sdrpp        não instala o SDR++
--skip-satdump      não instala o SatDump
--dry-run           mostra os comandos exatos, sem executar
--yes               não pergunta nada (exceto a senha do sudo)
--verbose           log em nível debug
```

### Depois de instalar

```bash
./status.sh          # o que está rodando, o que caiu, o que falta
viewadsb             # tabela ao vivo no terminal
nc localhost 30003   # mensagens decodificadas em CSV
```

E o mapa web em `http://IP-DO-SERVIDOR/tar1090/`.

## Configuração

A configuração vive em `install.conf`, gerado a partir de `install.conf.example` na primeira execução. É shell sourceável, no estilo `/etc/default/*`, sem depender de parser nenhum.

Como o instalador é idempotente, o arquivo funciona como declaração do estado desejado da máquina: rodar de novo converge para o que está escrito lá. Não existe modo `--update` separado, re-rodar **é** o update.

Dois pontos que merecem atenção antes da primeira execução:

`JSON_LOCATION_ACCURACY` controla a precisão da posição do seu receptor exposta no JSON e no mapa. O padrão é exata; se o mapa for acessível fora da sua rede, considere baixar para aproximada ou não publicar.

`FEEDER_ADSBEXCHANGE` vem desligado. Habilitar compartilha seus dados e sua posição com terceiros, então é opt-in por decisão, não por esquecimento. O instalador pergunta isso explicitamente na primeira execução.

O FlightAware não é oferecido de propósito: alimentar a rede deles exige o cliente `piaware`, com registro e ID de feeder próprios, e um `--net-connector` simples não alimenta nada.

## O que ele resolve por você

Esta lista é o valor real do projeto, e cada item custou depuração de verdade:

O `--noconfirm` do pacman responde "N" ao prompt de conflito, abortando a instalação em silêncio. Por isso a remoção de pacote conflitante é sempre um passo explícito, com confirmação.

O `yay` reseta qualquer edição no `PKGBUILD` do cache a cada execução, que é o comportamento correto contra adulteração, mas incompatível com patch manual. Builds que precisam de patch rodam num diretório copiado, fora de `~/.cache/yay/`.

O `blacklist` no modprobe.d só impede o autoload no boot. No hotplug o udev pede o módulo por alias e o kernel entrega assim mesmo. A linha `install <módulo> /bin/false` é o que fecha essa porta, e falta em quase todo tutorial.

A regra `udev` padrão libera o dongle para o grupo `plugdev`, o que basta para um usuário interativo porque o systemd-logind adiciona uma ACL de sessão. O usuário de serviço do readsb não tem sessão nem ACL, então precisa de uma regra dedicada ao grupo dele. O problema se esconde justamente porque funciona quando você testa na mão.

O `lighttpd.conf` do pacote Arch é minimalista e nunca inclui `conf-enabled`, então toda a configuração que o instalador do tar1090 grava lá fica morta, sem erro nenhum.

O Arch não carrega o `mod_redirect`, então o `url.redirect` do tar1090 é ignorado e a URL sem barra final devolve 404, que é justamente a URL anunciada pelo instalador ao terminar.

O instalador do tar1090 só reinicia o lighttpd se ele já estava rodando; um lighttpd recém-instalado continua parado e desabilitado.

O `sudo` não funciona sem tty real. O preflight detecta e falha cedo, com mensagem clara, em vez de deixar você descobrir no meio de um build de 45 minutos.

## Segurança

O script roda como usuário normal e aborta se for executado como root, porque `makepkg` e `yay` se recusam a rodar assim. A escalação é pontual, via `sudo`, só nos passos que precisam. Um `sudo -v` no início valida a senha uma vez e um keepalive em background mantém o timestamp vivo durante builds longos, para a senha não ser pedida no meio de uma compilação.

O instalador oficial do tar1090 é vendorizado em `vendor/`, não baixado da rede a cada execução, e é verificado por checksum antes de rodar. Atualizar significa baixar a versão nova, revisar o diff e trocar o pin em `install.conf`. Recusar-se a rodar um script de root alterado é proposital.

## Escopo

A v1 suporta Arch e derivados. A camada de gerenciador de pacotes está isolada em `lib/pkg-arch.sh`, então suportar outra distro é escrever um `pkg-debian.sh` com a mesma interface `pkg::*`, sem tocar nos módulos. Debian e Raspberry Pi OS não são prioridade porque já estão bem resolvidos pelos scripts oficiais do wiedehopf.

Não é um daemon nem um painel de configuração. É um instalador que roda quando você quiser.

## Estrutura

```
easy1090/
├── install.sh              entrypoint, orquestra os módulos
├── status.sh               status de cada componente (somente leitura)
├── install.conf.example    configuração de referência
├── lib/
│   ├── common.sh           log, execução com dry-run, sudo, config
│   ├── i18n.sh             seleção de idioma e tradução
│   ├── i18n/{pt,en}.sh     catálogos de mensagens
│   ├── pkg-arch.sh         camada de gerenciador de pacotes
│   ├── 00-preflight.sh     checagens read-only
│   ├── 10-driver.sh        fork da RTL-SDR Blog e blacklist do DVB
│   ├── 20-readsb.sh        decodificador, udev, systemd
│   ├── 30-tar1090.sh       mapa web e os ajustes do lighttpd no Arch
│   ├── 40-optional.sh      SDR++ e SatDump
│   └── 60-validate.sh      validação de ponta a ponta
└── vendor/
    └── tar1090-install.sh  instalador oficial, pinado por checksum
```

## Por trás das decisões

O passo a passo manual, com o porquê de cada escolha, está documentado na série sobre SDR do meu blog:

- [Capturando ADS-B em 1090 MHz com a RTL-SDR v4](https://esli.blog/posts/rtl-sdr-v4-adsb-1090/)
- [Do terminal ao mapa: ADS-B ao vivo na web com tar1090](https://esli.blog/posts/rtl-sdr-v4-tar1090/)
- [Guia prático: todas as formas de ver o ADS-B em tempo real](https://esli.blog/posts/guia-visualizacao-adsb/)

Se você quer entender antes de rodar um script de root, comece por lá.

## Estado

Em desenvolvimento. Testado em EndeavourOS e Omarchy com RTL-SDR Blog V4. Veja [KNOWN_ISSUES.md](KNOWN_ISSUES.md).

## Licença

MIT
