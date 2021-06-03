local util = import 'util/main.libsonnet';
local k = import 'ksonnet-util/kausal.libsonnet';

function(tag, namespace, envSlug=null, projectPathSlug=null)
  (util.inlineSpec('https://control.kube.cat:6443', namespace, envSlug, projectPathSlug))
  + {
    _config:: self.data._config,
    pastor:: self.data.pastor,
    data: (import 'pastor.libsonnet') + {
      config+:: {
        pastor+: {
          image+: {
            tag: tag,
          },
        },
      },
      pastor+: {
        pvc+:: util.volumeClaimTemplate.withStorageClass("ssd"),
        statefulset+:
          k.apps.v1.statefulSet.spec.template.metadata.withAnnotationsMixin(
            $.spec.resourceDefaults.annotations),
        ingress: util.ingressFor(super.service, "beta.c-v.sh", "c-v-sh-tls")
      },
    },
  }
