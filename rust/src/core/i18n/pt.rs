//! easy1090 message catalog (pt).
//!
//! Generated verbatim from lib/i18n/pt.sh by scripts/gen-catalogs.sh;
//! do not edit by hand. Key parity with the bash catalogs is enforced
//! in CI by scripts/i18n-parity.sh.

/// Keys appear in the bash catalog's order; lookups are linear, which
/// is fine at this size.
pub static MSG: &[(&str, &str)] = &[
    // Common
    ("yes_no", "[s/N] "),
    ("yes_chars", "SsYy"),
    ("cfg_missing_dry", "Config ainda não existe; seria criada a partir do exemplo."),
    ("cfg_created", "Config criada em %s (a partir do exemplo)."),
    ("cfg_example_missing", "Exemplo de config não encontrado: %s"),
    ("sudo_validating", "Validando sudo (a senha pode ser pedida agora)."),
    ("sudo_failed", "Não foi possível validar o sudo."),
    ("cmd_required", "Comando obrigatório não encontrado: %s"),
    // Receiver position
    ("pos_intro", "A posição da antena é usada para calcular alcance e distância das aeronaves."),
    ("pos_help_title", "Para descobrir suas coordenadas, use um destes:"),
    ("pos_help_osm", "  OpenStreetMap   https://www.openstreetmap.org   (botão direito no ponto, \"Mostrar endereço\")"),
    ("pos_help_gmaps", "  Google Maps     https://maps.google.com        (botão direito no ponto)"),
    ("pos_help_latlong", "  latlong.net     https://www.latlong.net        (busca por endereço)"),
    ("pos_help_tip", "Informe em graus decimais, com ponto. Sul e oeste são negativos."),
    ("pos_lat", "Latitude (ex: -23.58): "),
    ("pos_lon", "Longitude (ex: -46.55): "),
    ("pos_required", "Latitude e longitude são obrigatórias."),
    ("pos_required_yes", "RECEIVER_LAT/RECEIVER_LON são obrigatórios com --yes. Preencha %s."),
    ("pos_dry", "RECEIVER_LAT/RECEIVER_LON vazios; usaria valores informados na execução real."),
    ("pos_saved", "Posição gravada em %s"),
    ("pos_invalid", "Coordenadas inválidas: %s, %s. Use graus decimais com ponto, latitude entre -90 e 90, longitude entre -180 e 180."),
    // Data sharing
    ("feed_title", "Compartilhar seus dados com uma rede pública de rastreamento de voos?"),
    ("feed_explain", "Isso envia as aeronaves que você recebe, e a posição do seu receptor, para servidores de terceiros. Em troca, esses sites costumam liberar acesso premium a quem contribui."),
    ("feed_opt_none", "  1) Não compartilhar (padrão, tudo fica só na sua rede)"),
    ("feed_opt_adsbx", "  2) ADSBExchange (adsbexchange.com, sem filtro de aeronaves)"),
    ("feed_fa_note", "FlightAware não entra nesta lista: alimentar a rede deles exige o cliente piaware, com registro e ID de feeder próprios; um conector beast simples não funciona."),
    ("feed_prompt", "Escolha [1]: "),
    ("feed_none", "Feed local apenas; nada será compartilhado."),
    ("feed_enabled", "Feed habilitado: %s. Sua posição será compartilhada."),
    // Preflight
    ("pre_step", "Preflight"),
    ("pre_root", "Não rode como root. Use seu usuário normal; o script pede sudo quando precisa (makepkg e yay se recusam a rodar como root)."),
    ("pre_user_ok", "Usuário normal (uid %s)."),
    ("pre_tty_dry", "TTY: verificação relaxada em --dry-run."),
    ("pre_tty_missing", "Sem terminal interativo. O sudo precisa de um tty real; rode direto num terminal ou numa sessão SSH de verdade."),
    ("pre_tty_ok", "Terminal interativo disponível."),
    ("pre_osrelease", "/etc/os-release não encontrado; distro não identificada."),
    ("pre_distro_ok", "Distro compatível: %s"),
    ("pre_distro_bad", "A v1 suporta apenas Arch e derivados (detectado: %s). Outras distros estão no roadmap; veja o README."),
    ("pre_tools_missing", "Faltam comandos essenciais: %s"),
    ("pre_yay_missing", "yay não encontrado. Instale um helper de AUR antes de continuar (o easy1090 depende dele para readsb e demais pacotes do AUR)."),
    ("pre_tools_ok", "Ferramentas presentes: %s"),
    ("pre_lsusb_missing", "lsusb não encontrado (pacote usbutils); pulando a checagem do dongle."),
    ("pre_dongle_ok", "RTL-SDR detectada no barramento USB (%s)."),
    ("pre_dongle_missing", "Nenhuma RTL-SDR encontrada em lsusb (%s)."),
    ("pre_dongle_warn", "A instalação continua, mas nada vai decodificar sem o dongle conectado."),
    ("pre_dongle_confirm", "Seguir mesmo assim?"),
    ("pre_aborted", "Interrompido a pedido do usuário."),
    ("pre_done", "Preflight concluído."),
    // Driver
    ("drv_step", "Driver RTL-SDR"),
    ("drv_installed", "%s já instalado."),
    ("drv_conflict", "O pacote genérico %s conflita com o fork da RTL-SDR Blog."),
    ("drv_conflict_confirm", "Remover %s agora?"),
    ("drv_conflict_abort", "Sem remover o conflito, a instalação do fork falha."),
    ("drv_blacklist_ok", "Blacklist do %s já configurada."),
    ("drv_blacklist_set", "Configurando blacklist de %s"),
    ("drv_module_unload", "Descarregando %s (carregado agora)."),
    ("drv_module_unload_fail", "Não consegui descarregar %s; pode ser necessário reiniciar."),
    ("drv_module_absent", "%s não está carregado."),
    ("drv_test_missing", "rtl_test não encontrado no PATH; pulando validação do driver."),
    ("drv_test_running", "Validando o hardware com rtl_test."),
    ("drv_no_device", "Nenhum dispositivo suportado encontrado."),
    ("drv_no_device_hint", "Cheque o cabo e a porta USB (prefira as traseiras, ligadas direto à placa-mãe)."),
    ("drv_tuner_v4", "Tuner R828D detectado (RTL-SDR Blog V4)."),
    ("drv_tuner_v3", "Tuner R820T/R820T2 detectado (v3 ou clone)."),
    ("drv_tuner_unknown", "Dongle respondeu, mas não identifiquei o tuner. Saída completa em --verbose."),
    // readsb
    ("rsb_step", "readsb (decodificador ADS-B)"),
    ("rsb_installed", "%s já instalado."),
    ("rsb_conflict", "%s (fork Mictronics) conflita com %s e grava protobuf em vez de JSON."),
    ("rsb_conflict_confirm", "Remover %s agora?"),
    ("rsb_conflict_abort", "Os dois pacotes não convivem; sem remover não dá pra seguir."),
    ("rsb_legacy_override", "Override antigo do systemd encontrado (referencia $USER_OPTIONS, que não existe neste pacote)."),
    ("rsb_legacy_removed", "Override legado removido."),
    ("rsb_udev_ok", "Regra udev do readsb já existe."),
    ("rsb_udev_create", "Criando regra udev para o usuário de serviço readsb."),
    ("rsb_udev_comment", "# easy1090: entrega o dongle ao grupo do usuário de serviço readsb.\\n# A regra padrão usa GROUP=\"plugdev\", que não cobre um usuário sem sessão."),
    ("rsb_defaults_write", "Escrevendo %s"),
    ("rsb_defaults_header", "# Gerado pelo easy1090. Editável à vontade: o instalador só reescreve\\n# este arquivo quando você roda install.sh de novo."),
    ("rsb_enabling", "Habilitando e iniciando o serviço readsb."),
    ("rsb_active", "readsb ativo."),
    ("rsb_failed", "readsb não subiu. Veja: journalctl -u readsb -n 40 --no-pager"),
    ("rsb_json_ok", "JSON sendo gravado em %s"),
    ("rsb_json_wait", "%s ainda não existe; pode levar alguns segundos."),
    // tar1090
    ("tar_step", "tar1090 (mapa web ao vivo)"),
    ("tar_confd_ok", "%s já existe."),
    ("tar_confd_create", "Criando %s (o instalador do tar1090 procura por ele)."),
    ("tar_vendor_missing", "Instalador do tar1090 não encontrado em %s"),
    ("tar_pin_missing", "TAR1090_INSTALLER_SHA256 não definido na config."),
    ("tar_pin_current", "Checksum atual do arquivo vendorizado: %s"),
    ("tar_pin_hint", "Fixe esse valor na config para detectar alterações futuras."),
    ("tar_pin_mismatch", "Checksum do instalador do tar1090 não confere."),
    ("tar_pin_expected", "  esperado: %s"),
    ("tar_pin_got", "  obtido:   %s"),
    ("tar_pin_abort", "Recuse-se a rodar script de root alterado. Revise o arquivo antes de atualizar o pin."),
    ("tar_pin_ok", "Instalador vendorizado confere com o pin da config."),
    ("tar_service_ok", "Serviço tar1090 já habilitado."),
    ("tar_running", "Rodando o instalador oficial do tar1090 (vendorizado)."),
    ("tar_conf_missing", "%s não encontrado; pulando o ajuste do include."),
    ("tar_include_ok", "lighttpd.conf já inclui conf-enabled."),
    ("tar_include_add", "Adicionando o include de conf-enabled ao lighttpd.conf (o padrão do Arch não tem)."),
    ("tar_lighttpd_check", "Validando a configuração do lighttpd antes de subir."),
    ("tar_lighttpd_invalid", "Configuração do lighttpd inválida. Revise %s antes de continuar."),
    ("tar_redirect_ok", "mod_redirect já habilitado."),
    ("tar_redirect_add", "Habilitando mod_redirect (o tar1090 usa url.redirect, e o Arch não carrega esse módulo por padrão)."),
    ("tar_redirect_comment", "# easy1090: o tar1090 usa url.redirect para a URL sem barra final."),
    ("tar_lighttpd_enable", "Habilitando e iniciando o lighttpd."),
    ("tar_web_ok", "Mapa web respondendo em %s"),
    ("tar_web_slashless", "%s (sem barra final) devolveu %s; o redirect não está ativo."),
    ("tar_web_fail", "Mapa web devolveu HTTP %s."),
    ("tar_web_hint", "Cheque: systemctl status lighttpd tar1090"),
    // Optional
    ("opt_sdrpp_step", "SDR++ (visualizador de espectro)"),
    ("opt_sdrpp_note", "SDR++ não decodifica ADS-B; serve para conferir visualmente a energia RF em 1090 MHz."),
    ("opt_satdump_step", "SatDump (decodificador de satélites)"),
    ("opt_satdump_slow", "O build do SatDump é longo (cerca de 45 minutos no hardware de referência)."),
    ("opt_satdump_confirm", "Continuar com a instalação do SatDump?"),
    ("opt_satdump_skipped", "SatDump pulado."),
    // Validation
    ("val_step", "Validação final"),
    ("val_unit_missing", "Unidade não encontrada: %s.service"),
    ("val_service_ok", "%s: ativo e habilitado no boot."),
    ("val_service_bad", "%s: active=%s enabled=%s"),
    ("val_json_missing", "%s não existe. O readsb está gravando JSON?"),
    ("val_json_stale", "aircraft.json parado há %ss; o readsb pode ter travado."),
    ("val_decoding_ok", "readsb decodificando (JSON atualizado há %ss, %s aeronave(s) na tela)."),
    ("val_zero_aircraft", "Zero aeronaves agora é normal: depende de tráfego, antena e linha de visada."),
    ("val_web_bad", "http://localhost/tar1090/ devolveu %s."),
    ("val_web_ok", "tar1090 servindo mapa e dados."),
    ("val_web_data_bad", "O mapa responde, mas /tar1090/data/aircraft.json não. Cheque o serviço tar1090."),
    ("val_all_ok", "Tudo no ar."),
    ("val_failures", "%s verificação(ões) falharam."),
    ("val_howto", "Como acompanhar o tráfego:"),
    ("val_howto_viewadsb", "tabela ao vivo no terminal"),
    ("val_howto_nc", "mensagens decodificadas (SBS/CSV)"),
    ("val_howto_map", "mapa web ao vivo"),
    // CLI
    ("cli_dry_warning", "Modo --dry-run: nada será alterado; os comandos abaixo são os reais."),
    ("cli_unknown_opt", "Opção desconhecida: %s"),
    ("cli_usage", "easy1090 %s - instalador do stack ADS-B (Arch e derivados)

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
"),
    // status.sh
    ("sts_title", "status"),
    ("sts_hardware", "Hardware e driver"),
    ("sts_decoding", "Decodificação"),
    ("sts_web", "Web"),
    ("sts_optional", "Opcionais"),
    ("sts_running", "rodando"),
    ("sts_stopped", "parado"),
    ("sts_installed", "instalado"),
    ("sts_absent", "ausente"),
    ("sts_driver", "driver"),
    ("sts_service", "serviço"),
    ("sts_map", "mapa web"),
    ("sts_decode_row", "decodificação"),
    ("sts_lsusb_missing", "lsusb não instalado"),
    ("sts_dongle_found", "detectada no USB (%s)"),
    ("sts_dongle_absent", "nada em lsusb"),
    ("sts_unit_absent", "unidade não instalada"),
    ("sts_json_absent", "%s não existe"),
    ("sts_jq_missing", "jq não instalado"),
    ("sts_json_fresh", "JSON de %ss atrás, %s aeronave(s)"),
    ("sts_json_stale", "JSON parado há %ss"),
    ("sts_lighttpd_down", "lighttpd inativo"),
    ("sts_http", "HTTP %s"),
    // Restart / device busy
    ("rsb_restarting", "Config mudou; reiniciando o readsb para aplicar."),
    ("tar_lighttpd_restart", "Config mudou; reiniciando o lighttpd para aplicar."),
    ("drv_busy", "Dongle em uso pelo readsb (esperado numa reexecução); pulando o rtl_test."),
    ("drv_busy_v4", "Dongle em uso pelo readsb (RTL-SDR Blog V4); pulando o rtl_test."),
    // Packages
    ("pkg_installed", "Já instalado: %s"),
    ("pkg_pacman", "Instalando via pacman: %s"),
    ("pkg_absent", "Não instalado, nada a remover: %s"),
    ("pkg_removing", "Removendo pacote: %s"),
    ("pkg_aur", "Instalando via AUR: %s"),
    ("pkg_clean_build", "Limpando build anterior: %s"),
    ("pkg_cloning", "Clonando PKGBUILD de %s"),
    ("pkg_building", "Compilando e instalando (%s)"),
    ("pkg_build_missing", "Diretório de build não encontrado: %s"),
    // Uninstall
    ("un_title", "Desinstalação"),
    ("un_plan", "O que será removido:"),
    ("un_plan_driver", "  driver      pacote rtl-sdr-blog-git e a blacklist do módulo DVB"),
    ("un_plan_readsb", "  readsb      serviço, pacote, /etc/default/readsb e a regra udev"),
    ("un_plan_tar1090", "  tar1090     serviço, arquivos e as configs do lighttpd"),
    ("un_plan_optional", "  opcionais   SDR++ e SatDump (se instalados)"),
    ("un_plan_local", "  local       cache de build em ~/.cache/easy1090"),
    ("un_keep", "O que NÃO será tocado: lighttpd, jq, os usuários de sistema readsb e tar1090, e qualquer coisa que você tenha instalado por fora."),
    ("un_confirm", "Confirma a remoção?"),
    ("un_aborted", "Nada foi removido."),
    ("un_step_tar1090", "tar1090"),
    ("un_step_readsb", "readsb"),
    ("un_step_driver", "driver RTL-SDR"),
    ("un_step_optional", "opcionais"),
    ("un_nothing", "Nada a remover aqui."),
    ("un_step_local", "arquivos locais"),
    ("un_upstream", "Rodando o desinstalador do próprio tar1090 (%s)."),
    ("un_upstream_missing", "Desinstalador do tar1090 não encontrado; removendo o que o easy1090 criou."),
    ("un_include_removed", "Removida a linha de include que o easy1090 acrescentou ao lighttpd.conf."),
    ("un_lighttpd_restart", "Reiniciando o lighttpd."),
    ("un_stopping", "Parando e desabilitando %s."),
    ("un_removed", "Removido: %s"),
    ("un_absent", "Não existe, nada a fazer: %s"),
    ("un_pkg_kept", "Pacotes preservados (--keep-packages)."),
    ("un_config_ask", "Remover também o install.conf (suas coordenadas e preferências)?"),
    ("un_done", "Desinstalação concluída."),
    ("un_users_note", "Os usuários de sistema readsb e tar1090 continuam existindo; remova à mão com userdel se quiser."),
    ("un_usage", "easy1090 %s - desinstalador

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
"),
    // Entrypoint, services and open
    ("main_usage", "easy1090 %s - stack ADS-B em um comando (Arch e derivados)\\n\\nUSO\\n    easy1090 <comando> [opções]\\n\\nCOMANDOS\\n    install       instala o stack (idempotente, seguro reexecutar)\\n    update        atualiza versões dos pacotes (o install converge config)\\n    feed          alimenta redes públicas (ADSBExchange, airplanes.live)\\n    uninstall     desfaz a instalação (best-effort)\\n    status        o que está rodando, o que caiu, o que falta\\n    start         sobe readsb, lighttpd e tar1090\\n    stop          derruba os três\\n    restart       reinicia os três, na ordem certa\\n    open [alvo]   abre um componente (sem alvo, lista as opções)\\n\\nOPÇÕES GLOBAIS\\n    --lang <pt|en>   idioma da interface\\n    --dry-run        imprime os comandos exatos, sem executar\\n    --yes            não pergunta nada (exceto a senha do sudo)\\n    --verbose        log em nível debug\\n    --version        mostra a versão\\n    -h, --help       esta ajuda\\n\\nUse \"easy1090 <comando> --help\" para as opções de cada comando.\\n\\nstatus e open não precisam de sudo.\\n"),
    ("cmd_unknown", "Comando desconhecido: %s"),
    ("cmd_missing", "Informe um comando. Use --help para ver a lista."),
    ("svc_step", "Serviços"),
    ("svc_acting", "%s: %s"),
    ("svc_not_installed", "%s não está instalado; pulando."),
    ("svc_done", "Pronto."),
    ("open_step", "Abrir"),
    ("open_targets", "Alvos disponíveis:"),
    ("open_t_viewadsb", "  viewadsb    tabela ao vivo no terminal (ncurses)"),
    ("open_t_sbs", "  sbs         stream de mensagens decodificadas (CSV)"),
    ("open_t_map", "  map         mapa web no navegador"),
    ("open_t_sdrpp", "  sdrpp       SDR++ (interface gráfica)"),
    ("open_t_satdump", "  satdump     SatDump (interface gráfica)"),
    ("open_unknown", "Alvo desconhecido: %s"),
    ("open_missing", "Comando não encontrado: %s. O componente está instalado?"),
    ("open_no_display", "Sem sessão gráfica ($DISPLAY/$WAYLAND_DISPLAY vazios); não dá para abrir %s aqui."),
    ("open_url", "Mapa web: %s"),
    ("open_running", "Rodando: %s"),
    // Update
    ("upd_step_aur", "Pacotes do AUR rastreados pelo yay"),
    ("upd_step_readsb", "readsb (compilado fora do yay)"),
    ("upd_step_services", "Serviços"),
    ("upd_yay_note", "O --devel é obrigatório: pacote -git não muda de versão no AUR quando o upstream commita."),
    ("upd_not_installed", "%s não está instalado; nada a atualizar. Use \"easy1090 install\"."),
    ("upd_readsb_installed", "Instalado: commit %s"),
    ("upd_readsb_upstream", "Upstream: commit %s"),
    ("upd_readsb_current", "readsb já está no commit atual do upstream."),
    ("upd_readsb_behind", "Há commits novos no upstream."),
    ("upd_readsb_unknown", "Não consegui comparar os commits (sem rede ou formato de versão inesperado); pulando."),
    ("upd_readsb_confirm", "Recompilar o readsb a partir do HEAD?"),
    ("upd_readsb_skipped", "readsb mantido na versão atual."),
    ("upd_readsb_rebuilt", "readsb recompilado a partir do commit %s."),
    ("upd_services_restart", "Reiniciando %s para carregar os binários novos."),
    ("upd_services_ok", "Nada mudou; serviços não precisam reiniciar."),
    ("upd_done", "Atualização concluída."),
    ("upd_hint_install", "Rode \"easy1090 install\" se quiser reconvergir a configuração também."),
    ("upd_usage", "easy1090 %s - atualização

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
"),
    // Feed
    ("feed_step_cfg", "Feeds"),
    ("feed_step_stats", "Pacote de estatísticas"),
    ("feed_step_info", "Seu feeder"),
    ("feed_enabling", "Habilitando o feed. Sua posição e as aeronaves recebidas serão enviadas ao ADSBExchange."),
    ("feed_disabling", "Desabilitando o feed. Nada mais será compartilhado."),
    ("feed_already_on", "Feed já está habilitado na config."),
    ("feed_already_off", "Feed já está desabilitado."),
    ("feed_connected", "Conectado a %s"),
    ("feed_not_connected", "Sem conexão estabelecida com o ADSBExchange no momento."),
    ("feed_readsb_missing", "readsb ainda não está instalado; a escolha foi registrada no install.conf, mas o /etc/default/readsb não foi reescrito. Rode easy1090 install primeiro."),
    ("feed_stats_present", "Serviço adsbexchange-stats já instalado e habilitado."),
    ("feed_stats_intro", "O pacote de estatísticas é código de terceiro, do ADSBExchange, e o instalador dele roda como root:"),
    ("feed_stats_repo", "  %s"),
    ("feed_stats_note", "O script oficial não roda no Arch: ele usa adduser, que não existe aqui, e morre antes de instalar qualquer coisa. O easy1090 cria o usuário de sistema antes e resolve as dependências, depois entrega o resto ao instalador deles, sem alterá-lo."),
    ("feed_stats_confirm", "Instalar o pacote de estatísticas?"),
    ("feed_stats_skipped", "Pacote de estatísticas não instalado."),
    ("feed_stats_deps", "Garantindo as dependências do script (%s)."),
    ("feed_stats_user", "Criando o usuário de sistema adsbexchange (o script usa adduser, que não existe no Arch)."),
    ("feed_stats_user_ok", "Usuário adsbexchange já existe."),
    ("feed_stats_cloning", "Clonando o repositório de estatísticas."),
    ("feed_stats_running", "Rodando o instalador oficial do ADSBExchange."),
    ("feed_stats_ok", "Pacote de estatísticas instalado e ativo."),
    ("feed_stats_failed", "O instalador de estatísticas falhou. Veja: journalctl -u adsbexchange-stats -n 30"),
    ("feed_uuid", "UUID do feeder: %s"),
    ("feed_uuid_missing", "UUID ainda não gerado; o serviço de estatísticas cria na primeira execução."),
    ("feed_url_stats", "  Estatísticas do seu feeder:   %s"),
    ("feed_url_myip", "  Validar que está alimentando: https://adsbexchange.com/myip/"),
    ("feed_url_account", "  Vincular a uma conta:         https://account.adsbexchange.com/ (use o UUID acima)"),
    ("feed_privacy", "Lembre: com JSON_LOCATION_ACCURACY=%s, a posição publicada é %s."),
    ("feed_privacy_exact", "exata"),
    ("feed_privacy_approx", "aproximada"),
    ("feed_privacy_none", "não publicada"),
    ("feed_usage", "easy1090 %s - feeds de redes públicas\\n\\nUSO\\n    easy1090 feed [rede] [opções]\\n\\nSem argumentos: lista as redes e o estado de cada uma, sem alterar nada.\\n\\nREDES\\n    adsbexchange        habilita e instala o pacote de estatísticas deles\\n    airplaneslive       habilita (o readsb envia direto, sem pacote extra)\\n\\nDá para alimentar as duas ao mesmo tempo: são conexões independentes.\\n\\nOPÇÕES\\n    --status            só mostra o estado atual, não altera nada\\n    --disable           desabilita a rede informada\\n    --stats             instala ou repara só o pacote do ADSBExchange\\n    --dry-run           imprime os comandos exatos, sem executar\\n    --yes               não pergunta nada (exceto a senha do sudo)\\n    -h, --help          esta ajuda\\n\\nAlimentar envia o seu IP e as aeronaves que você recebe para terceiros.\\n"),
    ("feed_datasource_fix", "Apontando o stats para /run/readsb (por padrão ele só olha /run/adsbexchange-feed, do pacote de feed deles)."),
    ("feed_datasource_ok", "Fonte de dados do stats já configurada."),
    ("feed_datasource_restart", "Reiniciando o adsbexchange-stats para aplicar."),
    ("feed_datasource_working", "O stats está lendo os dados do readsb."),
    ("feed_datasource_wait", "O stats ainda não confirmou a leitura; veja: journalctl -u adsbexchange-stats -n 20"),
    ("feed_net_adsbx", "ADSBExchange"),
    ("feed_net_alive", "airplanes.live"),
    ("feed_list_title", "Redes disponíveis:"),
    ("feed_list_adsbx", "  adsbexchange   %s   não filtra aeronaves; vendida para a JETNET em 2023"),
    ("feed_list_alive", "  airplaneslive  %s   comunitária, sem filtro, criada depois daquela venda"),
    ("feed_list_on", "[ligado]   "),
    ("feed_list_off", "[desligado]"),
    ("feed_list_hint", "Para ligar ou desligar:\\n  easy1090 feed <rede>\\n  easy1090 feed <rede> --disable\\n\\nDá para alimentar as duas ao mesmo tempo: são conexões independentes."),
    ("feed_unknown_net", "Rede desconhecida: %s. Use adsbexchange ou airplaneslive."),
    ("feed_alive_enabling", "Habilitando o feed do airplanes.live. Sua posição não é enviada, mas o seu IP e as aeronaves recebidas, sim."),
    ("feed_alive_disabling", "Desabilitando o feed do airplanes.live."),
    ("feed_alive_note", "O airplanes.live não precisa de pacote extra para o ADS-B: o próprio readsb envia. O instalador oficial deles serve para quem também quer alimentar MLAT, o que exige cliente separado."),
    ("feed_alive_url", "  Status do seu feed:  https://airplanes.live/myfeed/"),
    ("feed_alive_map", "  Mapa da rede:        https://globe.airplanes.live/"),
    ("feed_opt_alive", "  3) airplanes.live (comunitária, sem filtro, criada depois da venda do ADSBExchange)"),
];
