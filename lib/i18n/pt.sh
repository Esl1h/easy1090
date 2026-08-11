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
MSG[main_usage]="easy1090 %s - stack ADS-B em um comando (Arch e derivados)\n\nUSO\n    easy1090 <comando> [opções]\n\nCOMANDOS\n    install       instala o stack (idempotente, seguro reexecutar)\n    update        atualiza versões dos pacotes (o install converge config)\n    feed          habilita e configura o feed do ADSBExchange\n    uninstall     desfaz a instalação (best-effort)\n    status        o que está rodando, o que caiu, o que falta\n    start         sobe readsb, lighttpd e tar1090\n    stop          derruba os três\n    restart       reinicia os três, na ordem certa\n    open [alvo]   abre um componente (sem alvo, lista as opções)\n\nOPÇÕES GLOBAIS\n    --lang <pt|en>   idioma da interface\n    --dry-run        imprime os comandos exatos, sem executar\n    --yes            não pergunta nada (exceto a senha do sudo)\n    --verbose        log em nível debug\n    --version        mostra a versão\n    -h, --help       esta ajuda\n\nUse \"easy1090 <comando> --help\" para as opções de cada comando.\n\nstatus e open não precisam de sudo.\n"
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

#-------------------------------------------------------------------------------
# Update
#-------------------------------------------------------------------------------
MSG[upd_step_aur]="Pacotes do AUR rastreados pelo yay"
MSG[upd_step_readsb]="readsb (compilado fora do yay)"
MSG[upd_step_services]="Serviços"
MSG[upd_yay_note]="O --devel é obrigatório: pacote -git não muda de versão no AUR quando o upstream commita."
MSG[upd_not_installed]="%s não está instalado; nada a atualizar. Use \"easy1090 install\"."
MSG[upd_readsb_installed]="Instalado: commit %s"
MSG[upd_readsb_upstream]="Upstream: commit %s"
MSG[upd_readsb_current]="readsb já está no commit atual do upstream."
MSG[upd_readsb_behind]="Há commits novos no upstream."
MSG[upd_readsb_unknown]="Não consegui comparar os commits (sem rede ou formato de versão inesperado); pulando."
MSG[upd_readsb_confirm]="Recompilar o readsb a partir do HEAD?"
MSG[upd_readsb_skipped]="readsb mantido na versão atual."
MSG[upd_readsb_rebuilt]="readsb recompilado a partir do commit %s."
MSG[upd_services_restart]="Reiniciando %s para carregar os binários novos."
MSG[upd_services_ok]="Nada mudou; serviços não precisam reiniciar."
MSG[upd_done]="Atualização concluída."
MSG[upd_hint_install]="Rode \"easy1090 install\" se quiser reconvergir a configuração também."
MSG[upd_usage]="easy1090 %s - atualização

USO
    easy1090 update [opções]

Atualiza as VERSÕES dos pacotes. Para convergir configuração e serviços,
use \"easy1090 install\".

O que ele faz:
  1. yay -Syu --devel nos pacotes AUR que o yay rastreia (driver, SDR++, SatDump)
  2. compara o commit do readsb com o HEAD do upstream e recompila se estiver atrás
  3. reinicia os serviços cujos binários mudaram

OPÇÕES
    --skip-aur          não roda o yay, atualiza só o readsb
    --skip-readsb       não mexe no readsb
    --dry-run           imprime os comandos exatos, sem executar
    --yes               não pergunta nada (exceto a senha do sudo)
    -h, --help          esta ajuda
"

#-------------------------------------------------------------------------------
# Feed
#-------------------------------------------------------------------------------
MSG[feed_step_cfg]="Feed do ADSBExchange"
MSG[feed_step_stats]="Pacote de estatísticas"
MSG[feed_step_info]="Seu feeder"
MSG[feed_enabling]="Habilitando o feed. Sua posição e as aeronaves recebidas serão enviadas ao ADSBExchange."
MSG[feed_disabling]="Desabilitando o feed. Nada mais será compartilhado."
MSG[feed_already_on]="Feed já está habilitado na config."
MSG[feed_already_off]="Feed já está desabilitado."
MSG[feed_connected]="Conectado a %s"
MSG[feed_not_connected]="Sem conexão estabelecida com o ADSBExchange no momento."
MSG[feed_stats_present]="Serviço adsbexchange-stats já instalado e habilitado."
MSG[feed_stats_intro]="O pacote de estatísticas é código de terceiro, do ADSBExchange, e o instalador dele roda como root:"
MSG[feed_stats_repo]="  %s"
MSG[feed_stats_note]="O script oficial não roda no Arch: ele usa adduser, que não existe aqui, e morre antes de instalar qualquer coisa. O easy1090 cria o usuário de sistema antes e resolve as dependências, depois entrega o resto ao instalador deles, sem alterá-lo."
MSG[feed_stats_confirm]="Instalar o pacote de estatísticas?"
MSG[feed_stats_skipped]="Pacote de estatísticas não instalado."
MSG[feed_stats_deps]="Garantindo as dependências do script (%s)."
MSG[feed_stats_user]="Criando o usuário de sistema adsbexchange (o script usa adduser, que não existe no Arch)."
MSG[feed_stats_user_ok]="Usuário adsbexchange já existe."
MSG[feed_stats_cloning]="Clonando o repositório de estatísticas."
MSG[feed_stats_running]="Rodando o instalador oficial do ADSBExchange."
MSG[feed_stats_ok]="Pacote de estatísticas instalado e ativo."
MSG[feed_stats_failed]="O instalador de estatísticas falhou. Veja: journalctl -u adsbexchange-stats -n 30"
MSG[feed_uuid]="UUID do feeder: %s"
MSG[feed_uuid_missing]="UUID ainda não gerado; o serviço de estatísticas cria na primeira execução."
MSG[feed_url_stats]="  Estatísticas do seu feeder:   %s"
MSG[feed_url_myip]="  Validar que está alimentando: https://adsbexchange.com/myip/"
MSG[feed_url_account]="  Vincular a uma conta:         https://account.adsbexchange.com/ (use o UUID acima)"
MSG[feed_privacy]="Lembre: com JSON_LOCATION_ACCURACY=%s, a posição publicada é %s."
MSG[feed_privacy_exact]="exata"
MSG[feed_privacy_approx]="aproximada"
MSG[feed_privacy_none]="não publicada"
MSG[feed_usage]="easy1090 %s - feed do ADSBExchange\n\nUSO\n    easy1090 feed [opções]\n\nSem opções: habilita o feed, converge a config do readsb e oferece instalar o\npacote de estatísticas do ADSBExchange.\n\nOPÇÕES\n    --status            só mostra o estado atual, não altera nada\n    --disable           desabilita o feed\n    --stats             instala ou repara só o pacote de estatísticas\n    --dry-run           imprime os comandos exatos, sem executar\n    --yes               não pergunta nada (exceto a senha do sudo)\n    -h, --help          esta ajuda\n\nCompartilhar envia a posição do seu receptor e as aeronaves que você recebe\npara servidores de terceiros.\n"
MSG[feed_datasource_fix]="Apontando o stats para /run/readsb (por padrão ele só olha /run/adsbexchange-feed, do pacote de feed deles)."
MSG[feed_datasource_ok]="Fonte de dados do stats já configurada."
MSG[feed_datasource_restart]="Reiniciando o adsbexchange-stats para aplicar."
MSG[feed_datasource_working]="O stats está lendo os dados do readsb."
MSG[feed_datasource_wait]="O stats ainda não confirmou a leitura; veja: journalctl -u adsbexchange-stats -n 20"
