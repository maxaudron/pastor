{
  lib,
  config,
  pkgs,
  ...
}:

let
  cfg = config.services.pastor;
in

with lib;
{
  options = {
    services.pastor = {
      enable = lib.mkEnableOption "pastor";

      package = mkOption {
        defaultText = lib.literalMD "`packages.default` from the pastor flake";
      };

      extraEnvironment = mkOption {
        type = types.attrsOf types.str;
        description = "Extra environment variables to pass to the service.";
        default = { };
        example = {
          RUST_BACKTRACE = "yes";
        };
      };

      environmentFile = mkOption {
        type = types.nullOr types.path;
        description = "File containing environment variables to be passed to the service.";
        default = null;
      };

      logLevel = mkOption {
        type = types.enum ([
          "error"
          "warn"
          "info"
          "debug"
          "trace"
        ]);
        default = "info";
        example = "debug";
      };

      address = mkOption {
        default = "127.0.0.1";
        type = types.str;
      };

      port = mkOption {
        default = 6881;
        type = types.int;
      };

      storage_dir = mkOption {
        default = "/var/lib/pastor/storage";
        type = types.path;
      };

      tokens_file = mkOption {
        default = "/var/lib/pastor/tokens.toml";
        type = types.path;
      };

      file_size_limit = mkOption {
        default = 256;
        type = types.int;
        description = "Maximum size for uploaded files in megabytes";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.pastor = {
      description = "Pastor: The Bestest Pastebin";
      after = [
        "network.target"
        "network-online.target"
      ];
      wants = [
        "network.target"
        "network-online.target"
      ];
      wantedBy = [ "multi-user.target" ];
      restartTriggers = lib.optional (cfg.environmentFile != null) cfg.environmentFile;
      serviceConfig = {
        ExecStart = lib.strings.concatStringsSep " " [
          "${cfg.package}/bin/pastor"
          "--address ${toString cfg.address}"
          "--port ${toString cfg.port}"
          "--tokens ${toString cfg.tokens_file}"
          "--storage ${toString cfg.storage_dir}"
          "--file-size-limit ${toString cfg.file_size_limit}"
        ];

        StateDirectory = mkIf (hasPrefix "/var/lib/pastor" cfg.storage_dir) "pastor";
        DynamicUser = lib.mkDefault true;
        ProtectHome = true;
        NoNewPrivileges = true;
        EnvironmentFile = lib.optional (cfg.environmentFile != null) cfg.environmentFile;
      };
      environment = {
        RUST_LOG = lib.mkDefault "pastor=${cfg.logLevel}";
        PASTOR_MIME_DB = "${pkgs.file}/share/misc/";
      }
      // cfg.extraEnvironment;
    };
  };
}
