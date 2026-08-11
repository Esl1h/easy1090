# Known issues

Changelog vivo dos atritos conhecidos. A regra aqui é documentar antes de corrigir: o histórico da decisão vale mais que o fix isolado, porque esses problemas voltam a cada release do GCC ou mudança de PKGBUILD no AUR.

## Ativos

### O build do readsb quebra com GCC novo (fork Mictronics)

O `readsb-git` (fork Mictronics) usa `-Werror` no Makefile e não recebe manutenção desde por volta de 2020. O GCC atual quebra o build de duas formas diferentes:

Primeiro, avisos novos que viram erro por causa do `-Werror` do próprio projeto, como `unterminated-string-initialization` e `format-truncation`. Contornável removendo a flag.

Segundo, e mais traiçoeiro, diagnósticos que o GCC 14 passou a tratar como erro **por padrão**, independentemente de `-Werror`: `incompatible-pointer-types`, `implicit-function-declaration`, `int-conversion` e `implicit-int`. Esses precisam de `-Wno-error=` explícito.

O easy1090 usa o fork wiedehopf, que hoje compila limpo, então o problema não aparece. Ficou registrado porque é uma classe de problema que atinge qualquer pacote AUR em C sem manutenção, e porque nada garante que o próximo GCC não faça o mesmo com o fork novo. Se isso acontecer, a correção é um `prepare()` no PKGBUILD, aplicado num diretório copiado (ver o item do yay abaixo).

### Instalação sem tty falha ou trava

Todo passo com `sudo` precisa de terminal real. Rodar via automação, CI ou ponte sem tty faz o `sudo` travar esperando senha ou falhar em silêncio. O preflight detecta e aborta cedo, mas vale repetir: o easy1090 não é feito para rodar sem interação humana no primeiro uso.

### Sem CI de verdade

Não dá para testar o caminho completo sem hardware RTL-SDR conectado. O que dá para automatizar é a parte estática: `shellcheck` e testes da lógica pura, como parsing de flags e detecção de distro. A validação de ponta a ponta continua manual, em VM ou máquina física, a cada mudança relevante do AUR ou do GCC.

## Resolvidos no código

Ficam aqui porque, se algum dia o comportamento upstream mudar, o motivo do código existir precisa estar escrito.

### pacman com --noconfirm assume o padrão errado em conflitos

O prompt de substituição de pacote conflitante é `[s/N]`, e `--noconfirm` responde N, abortando a instalação sem deixar claro o motivo. Por isso remoção de conflito é passo explícito e confirmado, nunca delegada ao `--noconfirm`.

### yay reseta o PKGBUILD do cache

Qualquer edição manual em `~/.cache/yay/<pkg>/PKGBUILD` é descartada na execução seguinte. É proteção correta contra adulteração, mas incompatível com patch manual. Builds que precisam de patch rodam num diretório próprio, fora do controle do yay.

### blacklist de módulo não impede carga por alias

`blacklist dvb_usb_rtl28xxu` no modprobe.d só impede o autoload no boot. No hotplug, o udev pede o módulo pelo alias e o kernel entrega, blacklist ou não. Só a diretiva `install dvb_usb_rtl28xxu /bin/false` fecha de fato. Falta em quase todo tutorial, e é a diferença entre um sistema que funciona e um que funciona até você replugar o dongle.

### Regra udev por grupo não cobre usuário de serviço

A regra padrão do `rtl-sdr` usa `GROUP="plugdev"`. Para o usuário interativo isso parece funcionar porque o systemd-logind concede uma ACL de sessão sobre o dispositivo, o que mascara o problema. O usuário de serviço do readsb não tem sessão, não recebe ACL e leva EACCES. A correção é uma regra dedicada ao grupo do serviço.

### lighttpd do Arch não carrega conf-enabled

O `lighttpd.conf` do pacote Arch tem só o essencial e não inclui nenhum diretório de configuração extra, diferente do Debian. Sem acrescentar o `include_shell`, toda a configuração que o instalador do tar1090 grava fica morta, silenciosamente, e o mapa responde 404 com tudo aparentemente instalado.

### Instalador do tar1090 não sobe um lighttpd novo

O script upstream só reinicia o lighttpd se ele já estava ativo antes. Um lighttpd recém-instalado permanece `inactive (dead)` e desabilitado, mesmo com tudo o mais configurado corretamente.
