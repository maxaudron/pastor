{
  _config:: {
    pastor: {
      name: 'pastor',
      image: {
        repo: 'kube.cat/cocainefarm/pastor',
        tag: 'latest',
      },
    },
  },

  local k = import 'ksonnet-util/kausal.libsonnet',
  local util = import 'util/main.libsonnet',

  local statefulset = k.apps.v1.statefulSet,
  local container = k.core.v1.container,
  local env = k.core.v1.envVar,
  local envVarSrc = k.core.v1.envVarSource,
  local port = k.core.v1.containerPort,
  local service = k.core.v1.service,

  local withEnv(name, value) = container.withEnv(
    env.new(name=name, value=value)
  ),

  pastor: {
    pvc:: util.volumeClaimTemplate.new('data', '50Gi'),
    statefulset:
      statefulset.new(
        name=$._config.pastor.name
        , replicas=1
        , containers=[
          container.new(
            'pastor'
            , util.image($._config.pastor.image)
          )
          + container.withPorts([port.new('http', 8000)])
          + k.util.resourcesRequests('10m', '150Mi')
          + util.httpProbes('http', '/', 30, 10, 5, 5)
          + container.withVolumeMounts(
            k.core.v1.volumeMount.new("data", "/storage", readOnly=false)
          )
        ]
      )
      + statefulset.spec.withVolumeClaimTemplates([$.pastor.pvc])
      + statefulset.spec.withServiceName(self.service.metadata.name),
    service: k.util.serviceFor(self.statefulset) + k.core.v1.service.spec.withClusterIP("None"),
  },
}
