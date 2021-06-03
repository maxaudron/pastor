local util(k) = {
  inlineSpec(apiserver, namespace, envSlug=null, projectPathSlug=null):: {
    apiVersion: 'tanka.dev/v1alpha1',
    kind: 'Environment',
    metadata: {
      name: 'environments/production',
    },
    spec: {
      apiServer: 'https://control.kube.cat:6443',
      namespace: namespace,
      resourceDefaults: {
        annotations+:
          if projectPathSlug != null then { 'app.gitlab.com/app': projectPathSlug }
          + if envSlug != null then { 'app.gitlab.com/env': envSlug },
      },
    },
  },

  injectGlobalAnnotations()::
    k.apps.v1.statefulSet.spec.template.metadata.withAnnotationsMixin(
      $.spec.resourceDefaults.annotations),

  httpProbes(port, path, livenessPeriod, livenessDelay, readinessPeriod, readinessDelay)::
    local container = k.core.v1.container;

    container.livenessProbe.httpGet.withPort(port)
    + container.livenessProbe.httpGet.withPath(path)
    + container.livenessProbe.withPeriodSeconds(livenessPeriod)
    + container.livenessProbe.withInitialDelaySeconds(livenessDelay)
    + container.readinessProbe.httpGet.withPort(port)
    + container.readinessProbe.httpGet.withPath(path)
    + container.readinessProbe.withPeriodSeconds(readinessPeriod)
    + container.readinessProbe.withInitialDelaySeconds(readinessDelay),

  mapVolumeClaimTemplates(func):: {
    local volumes = super.spec.volumeClaimTemplates,

    spec+: {
      volumeClaimTemplates: std.map(func, volumes)
    }
  },

  volumeClaimTemplate:: {
    new(name, size):: {
      metadata: {
        name: name,
      },
      spec: {
        resources: {
          requests: {
            storage: size,
          },
        },
        accessModes: [
          'ReadWriteOnce',
        ],
      },
    },

    withName(name):: {
      metadata+: {
        name: name,
      },
    },

    withAccessModes(accessModes):: {
      spec+: {
        accessModes: accessModes,
      },
    },

    withStorageClass(storageClass):: {
      spec+: {
        storageClassName: storageClass,
      },
    },

    withStorageRequests(request):: {
      spec+: {
        resources+: {
          requests+: {
            storage: request,
          },
        },
      },
    },
  },


  pvcVolumeMount(name, pvcName, path, readOnly=false, volumeMountMixin={})::
    local container = k.core.v1.container,
          deployment = k.apps.v1.deployment,
          volumeMount = k.core.v1.volumeMount,
          volume = k.core.v1.volume,
          addMount(c) = c + container.withVolumeMountsMixin(
      volumeMount.new(name, path, readOnly=readOnly) +
      volumeMountMixin,
    );

    deployment.mapContainers(addMount) +
    deployment.mixin.spec.template.spec.withVolumesMixin([
      volume.fromPersistentVolumeClaim(name, pvcName),
    ]),

  ingressFor(target, domain, tlsSecret)::
    local ingress = k.networking.v1.ingress,
          ingressRule = k.networking.v1.ingressRule,
          ingressTLS = k.networking.v1.ingressTLS,
          httpIngressPath = k.networking.v1.httpIngressPath,
          service = k.core.v1.service;

    ingress.new(target.metadata.name) +
    ingress.mixin.spec.withRules(
      ingressRule.withHost(domain) +
      ingressRule.mixin.http.withPaths(
        httpIngressPath.withPath('/') +
        httpIngressPath.withPathType('Prefix') +
        httpIngressPath.backend.service.withName(target.metadata.name) +
        httpIngressPath.backend.service.port.withName(target.spec.ports[0].name)
      )
    ) +
    ingress.mixin.spec.withTls(
      ingressTLS.withHosts(domain) +
      ingressTLS.withSecretName(tlsSecret)
    ),

  image(image)::
    image.repo + ':' + image.tag,
};

util((import 'k.libsonnet'))
