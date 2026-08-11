#!/usr/bin/env bash
#===============================================================================
# easy1090 - catálogo de mensagens (português)
#===============================================================================

# MSG é declarado como array associativo em lib/i18n.sh, que carrega este
# arquivo. Analisando o catálogo isoladamente, o shellcheck não enxerga essa
# declaração e acha que cada chave é uma variável solta.
# shellcheck disable=SC2034,SC2154

#-------------------------------------------------------------------------------
# Comum
#-------------------------------------------------------------------------------
MSG[yes_no]="[s/N] "
MSG[yes_chars]="SsYy"
MSG[cfg_missing_dry]="Config ainda não existe; seria criada a partir do exemplo."
MSG[cfg_created]="Config criada em %s (a partir do exemplo)."
MSG[cfg_example_missing]="Exemplo de config não encontrado: %s"
MSG[sudo_validating]="Validando sudo (a senha pode ser pedida agora)."
MSG[sudo_failed]="Não foi possível validar o sudo."
MSG[cmd_required]="Comando obrigatório não encontrado: %s"

#-------------------------------------------------------------------------------
# Posição do receptor
#-------------------------------------------------------------------------------
MSG[pos_intro]="A posição da antena é usada para calcular alcance e distância das aeronaves."
MSG[pos_help_title]="Para descobrir suas coordenadas, use um destes:"
MSG[pos_help_osm]="  OpenStreetMap   https://www.openstreetmap.org   (botão direito no ponto, \"Mostrar endereço\")"
MSG[pos_help_gmaps]="  Google Maps     https://maps.google.com        (botão direito no ponto)"
MSG[pos_help_latlong]="  latlong.net     https://www.latlong.net        (busca por endereço)"
MSG[pos_help_tip]="Informe em graus decimais, com ponto. Sul e oeste são negativos."
MSG[pos_lat]="Latitude (ex: -23.58): "
MSG[pos_lon]="Longitude (ex: -46.55): "
MSG[pos_required]="Latitude e longitude são obrigatórias."
MSG[pos_required_yes]="RECEIVER_LAT/RECEIVER_LON são obrigatórios com --yes. Preencha %s."
MSG[pos_dry]="RECEIVER_LAT/RECEIVER_LON vazios; usaria valores informados na execução real."
MSG[pos_saved]="Posição gravada em %s"

#-------------------------------------------------------------------------------
# Compartilhamento de dados
#-------------------------------------------------------------------------------
MSG[feed_title]="Compartilhar seus dados com uma rede pública de rastreamento de voos?"
MSG[feed_explain]="Isso envia as aeronaves que você recebe, e a posição do seu receptor, para servidores de terceiros. Em troca, esses sites costumam liberar acesso premium a quem contribui."
MSG[feed_opt_none]="  1) Não compartilhar (padrão, tudo fica só na sua rede)"
MSG[feed_opt_adsbx]="  2) ADSBExchange (adsbexchange.com, sem filtro de aeronaves)"
MSG[feed_fa_note]="FlightAware não entra nesta lista: alimentar a rede deles exige o cliente piaware, com registro e ID de feeder próprios; um conector beast simples não funciona."
MSG[feed_prompt]="Escolha [1]: "
MSG[feed_none]="Feed local apenas; nada será compartilhado."
MSG[feed_enabled]="Feed habilitado: %s. Sua posição será compartilhada."

#-------------------------------------------------------------------------------
# Preflight
#-------------------------------------------------------------------------------
MSG[pre_step]="Preflight"
MSG[pre_root]="Não rode como root. Use seu usuário normal; o script pede sudo quando precisa (makepkg e yay se recusam a rodar como root)."
MSG[pre_user_ok]="Usuário normal (uid %s)."
MSG[pre_tty_dry]="TTY: verificação relaxada em --dry-run."
MSG[pre_tty_missing]="Sem terminal interativo. O sudo precisa de um tty real; rode direto num terminal ou numa sessão SSH de verdade."
MSG[pre_tty_ok]="Terminal interativo disponível."
MSG[pre_osrelease]="/etc/os-release não encontrado; distro não identificada."
MSG[pre_distro_ok]="Distro compatível: %s"
MSG[pre_distro_bad]="A v1 suporta apenas Arch e derivados (detectado: %s). Outras distros estão no roadmap; veja o README."
MSG[pre_tools_missing]="Faltam comandos essenciais: %s"
MSG[pre_yay_missing]="yay não encontrado. Instale um helper de AUR antes de continuar (o easy1090 depende dele para readsb e demais pacotes do AUR)."
MSG[pre_tools_ok]="Ferramentas presentes: %s"
MSG[pre_lsusb_missing]="lsusb não encontrado (pacote usbutils); pulando a checagem do dongle."
MSG[pre_dongle_ok]="RTL-SDR detectada no barramento USB (%s)."
MSG[pre_dongle_missing]="Nenhuma RTL-SDR encontrada em lsusb (%s)."
MSG[pre_dongle_warn]="A instalação continua, mas nada vai decodificar sem o dongle conectado."
MSG[pre_dongle_confirm]="Seguir mesmo assim?"
MSG[pre_aborted]="Interrompido a pedido do usuário."
MSG[pre_done]="Preflight concluído."

#-------------------------------------------------------------------------------
# Driver
#-------------------------------------------------------------------------------
MSG[drv_step]="Driver RTL-SDR"
MSG[drv_installed]="%s já instalado."
MSG[drv_conflict]="O pacote genérico %s conflita com o fork da RTL-SDR Blog."
MSG[drv_conflict_confirm]="Remover %s agora?"
MSG[drv_conflict_abort]="Sem remover o conflito, a instalação do fork falha."
MSG[drv_blacklist_ok]="Blacklist do %s já configurada."
MSG[drv_blacklist_set]="Configurando blacklist de %s"
MSG[drv_module_unload]="Descarregando %s (carregado agora)."
MSG[drv_module_unload_fail]="Não consegui descarregar %s; pode ser necessário reiniciar."
MSG[drv_module_absent]="%s não está carregado."
MSG[drv_test_missing]="rtl_test não encontrado no PATH; pulando validação do driver."
MSG[drv_test_running]="Validando o hardware com rtl_test."
MSG[drv_no_device]="Nenhum dispositivo suportado encontrado."
MSG[drv_no_device_hint]="Cheque o cabo e a porta USB (prefira as traseiras, ligadas direto à placa-mãe)."
MSG[drv_tuner_v4]="Tuner R828D detectado (RTL-SDR Blog V4)."
MSG[drv_tuner_v3]="Tuner R820T/R820T2 detectado (v3 ou clone)."
MSG[drv_tuner_unknown]="Dongle respondeu, mas não identifiquei o tuner. Saída completa em --verbose."

#-------------------------------------------------------------------------------
# readsb
#-------------------------------------------------------------------------------
MSG[rsb_step]="readsb (decodificador ADS-B)"
MSG[rsb_installed]="%s já instalado."
MSG[rsb_conflict]="%s (fork Mictronics) conflita com %s e grava protobuf em vez de JSON."
MSG[rsb_conflict_confirm]="Remover %s agora?"
MSG[rsb_conflict_abort]="Os dois pacotes não convivem; sem remover não dá pra seguir."
MSG[rsb_legacy_override]="Override antigo do systemd encontrado (referencia \$USER_OPTIONS, que não existe neste pacote)."
MSG[rsb_legacy_removed]="Override legado removido."
MSG[rsb_udev_ok]="Regra udev do readsb já existe."
MSG[rsb_udev_create]="Criando regra udev para o usuário de serviço readsb."
MSG[rsb_udev_comment]="# easy1090: entrega o dongle ao grupo do usuário de serviço readsb.\n# A regra padrão usa GROUP=\"plugdev\", que não cobre um usuário sem sessão."
MSG[rsb_defaults_write]="Escrevendo %s"
MSG[rsb_defaults_header]="# Gerado pelo easy1090. Editável à vontade: o instalador só reescreve\n# este arquivo quando você roda install.sh de novo."
MSG[rsb_enabling]="Habilitando e iniciando o serviço readsb."
MSG[rsb_active]="readsb ativo."
MSG[rsb_failed]="readsb não subiu. Veja: journalctl -u readsb -n 40 --no-pager"
MSG[rsb_json_ok]="JSON sendo gravado em %s"
MSG[rsb_json_wait]="%s ainda não existe; pode levar alguns segundos."

#-------------------------------------------------------------------------------
# tar1090
#-------------------------------------------------------------------------------
MSG[tar_step]="tar1090 (mapa web ao vivo)"
MSG[tar_confd_ok]="%s já existe."
MSG[tar_confd_create]="Criando %s (o instalador do tar1090 procura por ele)."
MSG[tar_vendor_missing]="Instalador do tar1090 não encontrado em %s"
MSG[tar_pin_missing]="TAR1090_INSTALLER_SHA256 não definido na config."
MSG[tar_pin_current]="Checksum atual do arquivo vendorizado: %s"
MSG[tar_pin_hint]="Fixe esse valor na config para detectar alterações futuras."
MSG[tar_pin_mismatch]="Checksum do instalador do tar1090 não confere."
MSG[tar_pin_expected]="  esperado: %s"
MSG[tar_pin_got]="  obtido:   %s"
MSG[tar_pin_abort]="Recuse-se a rodar script de root alterado. Revise o arquivo antes de atualizar o pin."
MSG[tar_pin_ok]="Instalador vendorizado confere com o pin da config."
MSG[tar_service_ok]="Serviço tar1090 já habilitado."
MSG[tar_running]="Rodando o instalador oficial do tar1090 (vendorizado)."
MSG[tar_conf_missing]="%s não encontrado; pulando o ajuste do include."
MSG[tar_include_ok]="lighttpd.conf já inclui conf-enabled."
MSG[tar_include_add]="Adicionando o include de conf-enabled ao lighttpd.conf (o padrão do Arch não tem)."
MSG[tar_lighttpd_check]="Validando a configuração do lighttpd antes de subir."
MSG[tar_lighttpd_invalid]="Configuração do lighttpd inválida. Revise %s antes de continuar."
MSG[tar_redirect_ok]="mod_redirect já habilitado."
MSG[tar_redirect_add]="Habilitando mod_redirect (o tar1090 usa url.redirect, e o Arch não carrega esse módulo por padrão)."
MSG[tar_redirect_comment]="# easy1090: o tar1090 usa url.redirect para a URL sem barra final."
MSG[tar_lighttpd_enable]="Habilitando e iniciando o lighttpd."
MSG[tar_web_ok]="Mapa web respondendo em %s"
MSG[tar_web_slashless]="%s (sem barra final) devolveu %s; o redirect não está ativo."
MSG[tar_web_fail]="Mapa web devolveu HTTP %s."
MSG[tar_web_hint]="Cheque: systemctl status lighttpd tar1090"

#-------------------------------------------------------------------------------
# Opcionais
#-------------------------------------------------------------------------------
MSG[opt_sdrpp_step]="SDR++ (visualizador de espectro)"
MSG[opt_sdrpp_note]="SDR++ não decodifica ADS-B; serve para conferir visualmente a energia RF em 1090 MHz."
MSG[opt_satdump_step]="SatDump (decodificador de satélites)"
MSG[opt_satdump_slow]="O build do SatDump é longo (cerca de 45 minutos no hardware de referência)."
MSG[opt_satdump_confirm]="Continuar com a instalação do SatDump?"
MSG[opt_satdump_skipped]="SatDump pulado."

#-------------------------------------------------------------------------------
# Validação
#-------------------------------------------------------------------------------
MSG[val_step]="Validação final"
MSG[val_unit_missing]="Unidade não encontrada: %s.service"
MSG[val_service_ok]="%s: ativo e habilitado no boot."
MSG[val_service_bad]="%s: active=%s enabled=%s"
MSG[val_json_missing]="%s não existe. O readsb está gravando JSON?"
MSG[val_json_stale]="aircraft.json parado há %ss; o readsb pode ter travado."
MSG[val_decoding_ok]="readsb decodificando (JSON atualizado há %ss, %s aeronave(s) na tela)."
MSG[val_zero_aircraft]="Zero aeronaves agora é normal: depende de tráfego, antena e linha de visada."
MSG[val_web_bad]="http://localhost/tar1090/ devolveu %s."
MSG[val_web_ok]="tar1090 servindo mapa e dados."
MSG[val_web_data_bad]="O mapa responde, mas /tar1090/data/aircraft.json não. Cheque o serviço tar1090."
MSG[val_all_ok]="Tudo no ar."
MSG[val_failures]="%s verificação(ões) falharam."
MSG[val_howto]="Como acompanhar o tráfego:"
MSG[val_howto_viewadsb]="tabela ao vivo no terminal"
MSG[val_howto_nc]="mensagens decodificadas (SBS/CSV)"
MSG[val_howto_map]="mapa web ao vivo"

#-------------------------------------------------------------------------------
# CLI
#-------------------------------------------------------------------------------
MSG[cli_dry_warning]="Modo --dry-run: nada será alterado; os comandos abaixo são os reais."
MSG[cli_unknown_opt]="Opção desconhecida: %s"
MSG[cli_usage]="easy1090 %s - instalador do stack ADS-B (Arch e derivados)

USO
    ./install.sh [opções]

OPÇÕES
    --full              instala tudo, inclusive SDR++ e SatDump
    --lang <pt|en>      idioma da interface
    --lat <graus>       latitude da antena (ex: -23.58)
    --lon <graus>       longitude da antena (ex: -46.55)
    --skip-tar1090      não instala o mapa web
    --skip-sdrpp        não instala o SDR++
    --skip-satdump      não instala o SatDump
    --dry-run           roda o preflight e imprime os comandos exatos, sem executar
    --yes               não pergunta nada (exceto a senha do sudo)
    --verbose           log em nível debug
    --version           mostra a versão
    -h, --help          esta ajuda

EXEMPLOS
    ./install.sh                                  interativo, pergunta lat/lon
    ./install.sh --lat -23.58 --lon -46.55 --yes  sem interação
    ./install.sh --dry-run                        mostra o que faria

A configuração vive em install.conf (gerada a partir do .example na primeira
execução). As flags acima sobrescrevem o que estiver lá.
"

#-------------------------------------------------------------------------------
# status.sh
#-------------------------------------------------------------------------------
MSG[sts_title]="status"
MSG[sts_hardware]="Hardware e driver"
MSG[sts_decoding]="Decodificação"
MSG[sts_web]="Web"
MSG[sts_optional]="Opcionais"
MSG[sts_running]="rodando"
MSG[sts_stopped]="parado"
MSG[sts_installed]="instalado"
MSG[sts_absent]="ausente"
MSG[sts_driver]="driver"
MSG[sts_service]="serviço"
MSG[sts_map]="mapa web"
MSG[sts_decode_row]="decodificação"
MSG[sts_lsusb_missing]="lsusb não instalado"
MSG[sts_dongle_found]="detectada no USB (%s)"
MSG[sts_dongle_absent]="nada em lsusb"
MSG[sts_unit_absent]="unidade não instalada"
MSG[sts_json_absent]="%s não existe"
MSG[sts_jq_missing]="jq não instalado"
MSG[sts_json_fresh]="JSON de %ss atrás, %s aeronave(s)"
MSG[sts_json_stale]="JSON parado há %ss"
MSG[sts_lighttpd_down]="lighttpd inativo"
MSG[sts_http]="HTTP %s"

#-------------------------------------------------------------------------------
# Restart / device busy
#-------------------------------------------------------------------------------
MSG[rsb_restarting]="Config mudou; reiniciando o readsb para aplicar."
MSG[tar_lighttpd_restart]="Config mudou; reiniciando o lighttpd para aplicar."
MSG[drv_busy]="Dongle em uso pelo readsb (esperado numa reexecução); pulando o rtl_test."
MSG[drv_busy_v4]="Dongle em uso pelo readsb (RTL-SDR Blog V4); pulando o rtl_test."

#-------------------------------------------------------------------------------
# Packages
#-------------------------------------------------------------------------------
MSG[pkg_installed]="Já instalado: %s"
MSG[pkg_pacman]="Instalando via pacman: %s"
MSG[pkg_absent]="Não instalado, nada a remover: %s"
MSG[pkg_removing]="Removendo pacote: %s"
MSG[pkg_aur]="Instalando via AUR: %s"
MSG[pkg_clean_build]="Limpando build anterior: %s"
MSG[pkg_cloning]="Clonando PKGBUILD de %s"
MSG[pkg_building]="Compilando e instalando (%s)"
MSG[pkg_build_missing]="Diretório de build não encontrado: %s"

#-------------------------------------------------------------------------------
# Uninstall
#-------------------------------------------------------------------------------
MSG[un_title]="Desinstalação"
MSG[un_plan]="O que será removido:"
MSG[un_plan_driver]="  driver      pacote rtl-sdr-blog-git e a blacklist do módulo DVB"
MSG[un_plan_readsb]="  readsb      serviço, pacote, /etc/default/readsb e a regra udev"
MSG[un_plan_tar1090]="  tar1090     serviço, arquivos e as configs do lighttpd"
MSG[un_plan_optional]="  opcionais   SDR++ e SatDump (se instalados)"
MSG[un_plan_local]="  local       cache de build em ~/.cache/easy1090"
MSG[un_keep]="O que NÃO será tocado: lighttpd, jq, os usuários de sistema readsb e tar1090, e qualquer coisa que você tenha instalado por fora."
MSG[un_confirm]="Confirma a remoção?"
MSG[un_aborted]="Nada foi removido."
MSG[un_step_tar1090]="tar1090"
MSG[un_step_readsb]="readsb"
MSG[un_step_driver]="driver RTL-SDR"
MSG[un_step_optional]="opcionais"
MSG[un_nothing]="Nada a remover aqui."
MSG[un_step_local]="arquivos locais"
MSG[un_upstream]="Rodando o desinstalador do próprio tar1090 (%s)."
MSG[un_upstream_missing]="Desinstalador do tar1090 não encontrado; removendo o que o easy1090 criou."
MSG[un_include_removed]="Removida a linha de include que o easy1090 acrescentou ao lighttpd.conf."
MSG[un_lighttpd_restart]="Reiniciando o lighttpd."
MSG[un_stopping]="Parando e desabilitando %s."
MSG[un_removed]="Removido: %s"
MSG[un_absent]="Não existe, nada a fazer: %s"
MSG[un_pkg_kept]="Pacotes preservados (--keep-packages)."
MSG[un_config_ask]="Remover também o install.conf (suas coordenadas e preferências)?"
MSG[un_done]="Desinstalação concluída."
MSG[un_users_note]="Os usuários de sistema readsb e tar1090 continuam existindo; remova à mão com userdel se quiser."
MSG[un_usage]="easy1090 %s - desinstalador

USO
    ./uninstall.sh [opções]

OPÇÕES
    --keep-packages     remove serviços e configs, mas mantém os pacotes
    --lang <pt|en>      idioma da interface
    --dry-run           imprime os comandos exatos, sem executar
    --yes               não pergunta nada (exceto a senha do sudo)
    --verbose           log em nível debug
    -h, --help          esta ajuda

Não remove lighttpd nem jq, que são pacotes de uso geral.
"

#-------------------------------------------------------------------------------
# Entrypoint, services and open
#-------------------------------------------------------------------------------
MSG[main_usage]="easy1090 %s - stack ADS-B em um comando (Arch e derivados)\n\nUSO\n    easy1090 <comando> [opções]\n\nCOMANDOS\n    install       instala o stack (idempotente, seguro reexecutar)\n    uninstall     desfaz a instalação (best-effort)\n    status        o que está rodando, o que caiu, o que falta\n    start         sobe readsb, lighttpd e tar1090\n    stop          derruba os três\n    restart       reinicia os três, na ordem certa\n    open [alvo]   abre um componente (sem alvo, lista as opções)\n\nOPÇÕES GLOBAIS\n    --lang <pt|en>   idioma da interface\n    --dry-run        imprime os comandos exatos, sem executar\n    --yes            não pergunta nada (exceto a senha do sudo)\n    --verbose        log em nível debug\n    --version        mostra a versão\n    -h, --help       esta ajuda\n\nUse \"easy1090 <comando> --help\" para as opções de cada comando.\n\nstatus e open não precisam de sudo.\n"
MSG[cmd_unknown]="Comando desconhecido: %s"
MSG[cmd_missing]="Informe um comando. Use --help para ver a lista."
MSG[svc_step]="Serviços"
MSG[svc_acting]="%s: %s"
MSG[svc_not_installed]="%s não está instalado; pulando."
MSG[svc_done]="Pronto."
MSG[open_step]="Abrir"
MSG[open_targets]="Alvos disponíveis:"
MSG[open_t_viewadsb]="  viewadsb    tabela ao vivo no terminal (ncurses)"
MSG[open_t_sbs]="  sbs         stream de mensagens decodificadas (CSV)"
MSG[open_t_map]="  map         mapa web no navegador"
MSG[open_t_sdrpp]="  sdrpp       SDR++ (interface gráfica)"
MSG[open_t_satdump]="  satdump     SatDump (interface gráfica)"
MSG[open_unknown]="Alvo desconhecido: %s"
MSG[open_missing]="Comando não encontrado: %s. O componente está instalado?"
MSG[open_no_display]="Sem sessão gráfica (\$DISPLAY/\$WAYLAND_DISPLAY vazios); não dá para abrir %s aqui."
MSG[open_url]="Mapa web: %s"
MSG[open_running]="Rodando: %s"
