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

### O instalador do tar1090 imprime "adduser: command not found"

Duas vezes, logo no início. É ruído, não falha: a linha upstream é uma cadeia de fallback, `adduser ... || adduser ... || useradd -r -d "$ipath" -M tar1090`. As duas primeiras formas são Debianismos e não existem no Arch, então o `useradd` final é quem cria o usuário. Confirmado em teste real: o usuário `tar1090` é criado normalmente. Nada a corrigir do nosso lado, mas fica documentado para ninguém se assustar com a mensagem.

### lighttpd avisa "unknown config-key: url.redirect (ignored)"

Sintoma do mod_redirect ausente, descrito na seção de resolvidos. Se a mensagem voltar a aparecer depois de uma atualização, é sinal de que o arquivo `06-mod_redirect.conf` sumiu ou deixou de ser carregado.

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

### mod_redirect não é carregado no Arch, e a URL sem barra final dá 404

O `88-tar1090.conf` gerado pelo instalador usa `url.redirect` para mandar `/tar1090` para `/tar1090/`, mas o instalador só cria carregadores para `mod_alias` e `mod_setenv`. No Debian o `mod_redirect` já vem ligado na configuração base; no Arch não. O lighttpd então avisa `unknown config-key: url.redirect (ignored)` e segue, e a URL sem barra final devolve 404. Pior: é exatamente essa a URL que o instalador anuncia ao terminar. O easy1090 cria um `06-mod_redirect.conf` em `conf-available` e o habilita por symlink.

### Instalador do tar1090 não sobe um lighttpd novo

O script upstream só reinicia o lighttpd se ele já estava ativo antes. Um lighttpd recém-instalado permanece `inactive (dead)` e desabilitado, mesmo com tudo o mais configurado corretamente.

### `systemctl enable --now` não aplica config nova em serviço já rodando

Descoberto no segundo teste real. O instalador escrevia `/etc/default/readsb` com coordenadas novas e o `enable --now` não fazia nada, porque a unidade já estava ativa. Resultado: o daemon seguia rodando com a configuração anterior, sem erro nenhum, e o `journalctl` mostrava a lat/lon antiga enquanto o arquivo em disco tinha a nova. O mesmo valia para o lighttpd depois de habilitar o `mod_redirect`.

A correção é o `run::sudo_write` sinalizar se o conteúdo mudou de fato (`FILE_CHANGED`), e os módulos fazerem `restart` explícito quando mudou. É a mesma armadilha que já estava documentada no instalador upstream do tar1090, e que acabei repetindo.

### `rtl_test` falha quando o readsb está rodando

Numa reexecução, o `readsb` já detém o dispositivo, então o `rtl_test` enumera a placa mas não consegue reivindicar a interface, terminando com `usb_claim_interface error -6`. Isso não é defeito: é o sistema saudável. O módulo do driver reconhece essa saída e pula o teste, em vez de avisar que não identificou o tuner.

### Arquivo de config existir não significa que o serviço o carregou

Continuação do item anterior, e a parte que a primeira correção não cobriu. Na terceira execução real o `mod_redirect` aparecia como `[SKIP] já habilitado`, porque o arquivo e o symlink existiam, mas a URL sem barra final continuava devolvendo 404. O motivo: o arquivo foi criado às 15:45 por uma execução anterior que não reiniciou o lighttpd, e o daemon estava no ar desde 15:39. Presença do arquivo não prova que o processo o leu.

A correção é o `svc::predates_file`, que compara o `ActiveEnterTimestamp` da unidade com o mtime do arquivo. Se o serviço subiu antes do arquivo ser escrito, ele não pode tê-lo carregado, e o restart é agendado mesmo no caminho de `[SKIP]`.

### `--removemake` do yay quebra dependências opcionais

Descoberto testando `--full` numa máquina limpa. O easy1090 chamava o yay com `--removemake`, que descarta os pacotes instalados como dependência de build depois que o build termina. Parece limpeza sensata, e é exatamente o oposto disso.

Pacotes do AUR frequentemente listam a mesma biblioteca em `makedepends` e em `optdepends`: ela é necessária para compilar um plugin e necessária de novo, em runtime, para carregá-lo. O yay enxerga só o lado de build e remove. O programa continua instalado, os plugins continuam no disco, e nada reclama.

Medido no `sdrpp-git`: 24 pacotes removidos ao final da instalação, deixando 10 plugins sem suas bibliotecas, entre eles o `audio_sink.so`, ou seja, o SDR++ ficou sem saída de áudio. Nenhuma mensagem de erro em lugar nenhum.

A correção é não usar `--removemake`. O custo é deixar o ferramental de build no disco, recuperável com `yay -Yc` por quem se importar mais com espaço do que com os plugins funcionando.
