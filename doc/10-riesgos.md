\chapter{Análisis de Riesgos}

TODO

<!--
  En el caso de desarrollos orientados al usuario, debería analizarse el riesgo
  de aceptación y el nivel de satisfacción que podremos medir tras la
  implantación.

  En caso de desarrollos orientados a la investigación, se deberán analizar
  riesgos tecnológicos. Si, además, el resultado del TFG va a ser integrado en
  un proyecto de mayor alcance, también deberán considerarse los riesgos
  potenciales de dicha integración.

  Para analizar los riesgos, se proponen las siguientes herramientas:
    * Matriz de riesgos: Se puede encontrar un modelo en la siguiente imagen [..]
    * Análisis DAFO
    * Registro de riesgos mediante la siguiente tabla:
      * Número de riesgo
      * Descripción
      * Prioridad (Alta/Media/Baja)
      * Complejidad (Alta/Media/Baja)
      * Medidas correctoras
-->

Este capítulo presenta el análisis de riesgos del proyecto. Se analizan los
riesgos de aceptación, tecnológicos y de integración.

Se presenta a continuación una descripción detallada de los identificados en
este análisis, incluyendo su probabilidad e impacto, así como las medidas
correctivas o de mitigación propuestas.

**R1**             **Errores de sincronización**
-----------------  --------------------------------------------------------
**Descripción**    Es probable que en condiciones de red más inestables se
                   descubran errores
**Probabilidad**   Media
**Prioridad**      Alta
**Mitigaciones**   Mejores tests automáticos. Para errores no reparables, mejor
                   gestión de reconexiones

: R1: Errores de sincronización

**R2**             **Falta de adopción**
-----------------  --------------------------------------------------------
**Descripción**    No se obtiene la adopción esperada de esta biblioteca.
**Probabilidad**   Alta
**Prioridad**      Alta
**Mitigaciones**   Implementación en otras plataforms y de otras capas de
                   transporte.

: R2: Falta de adopción

**R3**             **Dificultad de mantenimiento**
-----------------  --------------------------------------------------------
**Descripción**    Las abstracciones entre plataformas y capas de transporte
                   pueden volverse difíciles de mantener con el tiempo.
**Probabilidad**   Alta
**Prioridad**      Media
**Mitigaciones**   Intentar mantener tanto código como sea posible en la capa
                   común. Tests de integración en diferentes plataformas.

: R3: Dificultad de mantenimiento

