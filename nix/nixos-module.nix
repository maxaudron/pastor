{ lib, config, pkgs, ... }:

let
  cfg = config.services.pastor;
  toml = pkgs.formats.toml { };
  configFile = toml.generate "config.toml" cfg.settings;
in

with lib;
{
  options = {
    services.pastor = {
      package = mkOption {
        defaultText = lib.literalMD "`packages.default` from the pastor flake";
      };

      extraEnvironment = mkOption {
        type = types.attrsOf types.str;
        description = "Extra environment variables to pass to the Garage server.";
        default = { };
        example = { RUST_BACKTRACE = "yes"; };
      };

      environmentFile = mkOption {
        type = types.nullOr types.path;
        description = "File containing environment variables to be passed to the Garage server.";
        default = null;
      };

      logLevel = mkOption {
        type = types.enum ([ "error" "warn" "info" "debug" "trace" ]);
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
        default = "/var/lib/pastor";
        type = types.path;
      };

      settings = mkOption {
        type = types.submodule {
          freeformType = toml.type;
        };
      };
    };
  };

  config = {
    services.pastor.settings = {
      default = {
        address = cfg.address;
        port = cfg.port;
        storage_dir = cfg.storage_dir;

        limits = { forms = lib.mkDefault "10 GB"; data-form = lib.mkDefault "10 GB"; };
      };
    };

    systemd.services.pastor = {
      description = "Pastor: The Bestest Pastebin";
      after = [ "network.target" "network-online.target" ];
      wants = [ "network.target" "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      restartTriggers = [ configFile ] ++ (lib.optional (cfg.environmentFile != null) cfg.environmentFile);
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/pastor";

        StateDirectory = mkIf (hasPrefix "/var/lib/pastor" cfg.storage_dir) "pastor";
        DynamicUser = lib.mkDefault true;
        ProtectHome = true;
        NoNewPrivileges = true;
        EnvironmentFile = lib.optional (cfg.environmentFile != null) cfg.environmentFile;
      };
      environment = {
        RUST_LOG = lib.mkDefault "pastor=${cfg.logLevel}";
        ROCKET_CONFIG = configFile;
      } // cfg.extraEnvironment;
    };
  };
}
