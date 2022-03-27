local util = import 'util/main.libsonnet';
local k = import 'ksonnet-util/kausal.libsonnet';

function(tag, namespace, envSlug=null, projectPathSlug=null, apiserver='https://kube.vapor.systems:6443')
  (util.inlineSpec(apiserver, namespace, envSlug, projectPathSlug))
  + {
    _config:: self.data._config,
    pastor:: self.data.pastor,
    data: (import 'pastor.libsonnet') + {
      _config+:: {
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
        ingress: util.ingressFor(super.service, "c-v.sh", "c-v-sh-tls")
          + k.networking.v1.ingress.spec.withIngressClassName('cdn')
          + k.networking.v1.ingress.metadata.withAnnotationsMixin({
                "kubernetes.io/tls-acme": "true",
                "nginx.ingress.kubernetes.io/proxy-body-size": "4G"
          })
      },
    },
  }
