#!/usr/bin/env bash
PRG="$0"
while [ -h "$PRG" ]; do
  ls=$(ls -ld "$PRG")
  link=$(expr "$ls" : '.*-> \(.*\)$')
  if expr "$link" : '/.*' >/dev/null; then
    PRG="$link"
  else
    PRG=$(dirname "$PRG")/"$link"
  fi
done
PRGDIR=$(dirname "$PRG")
_log() {
  ts=$(date "+%Y-%m-%d %H:%M:%S")
  echo "${ts} $@"
}
info_log() {
  _log "[INFO]" $@
}
warn_log() {
  _log "[WARN]" $@
}
error_log() {
  _log "[ERROR]" $@
}

# Parsing properties
parse_conf() {
  CONF_FILE=$1
  if [ -f $CONF_FILE ]; then
    info_log "Start to parsing ${1} ..."
    while IFS='=' read -r key value; do
      env_key=$(echo $key | tr .-/ _ | tr -cd 'A-Za-z0-9_')
      if [ "$value" != '' ]; then
        eval "export $env_key=$value"
        info_log "export ${env_key}=${value}"
      fi
    done <${CONF_FILE}
  else
    echo "$1 not found."
    exit 1
  fi
}

info_log ">>>>> ${0} - Program directory: ${PRGDIR}"
[ -z "$PROJECT_HOME" ] && export PROJECT_HOME=$(
  cd "${PRGDIR}" >/dev/null
  pwd
)

parse_conf ${PROJECT_HOME}/project.properties

info_log ">>>>> ${0} - Program directory: ${PRGDIR}"

# Parse options
SKIP_MVM=0
DIST=0
GEN_CODE=0
for arg in "$@"; do
  case $arg in
  "--gen-code")
    GEN_CODE=1
    shift
    ;;
  "--dist")
    DIST=1
    shift
    ;;
  "--skip-mvn")
    SKIP_MVM=1
    shift
    ;;
  esac
done

# Format code
ktlint -F "${PROJECT_HOME}/**/*.kt" >/dev/null

if [[ ${SKIP_MVM} != 1 ]]; then
  info_log " ============================ start compile!!! ============================== "
  if [[ ${GEN_CODE} == 1 ]]; then
    mvn clean dependency:sources install -Pgen-code -Dmaven.test.skip=true -Djooq.codegen.jdbc.driver=com.mysql.jdbc.Driver -Djooq.codegen.jdbc.url=jdbc:mysql://139.9.251.211:3306/nirvana_ops -Djooq.codegen.jdbc.user=bp_dev -Djooq.codegen.jdbc.password=@zdaBIF6sjIpwx45
  else
    mvn clean dependency:sources install -Dmaven.test.skip=true
  fi
  info_log " ============================ compile done!!! ============================== "
fi
if [[ ${DIST} == 1 ]]; then
  info_log " ============================ start deploy!!! ============================== "
  mvn deploy -Djooq.codegen.skip=true
  info_log " ============================ deploy done!!! ============================== "
fi
