\chapter{Estimación del tamaño y esfuerzo}

En el presente anexo se recoge la metodología de trabajo empleada en el
desarrollo del proyecto, la planificación temporal de las tareas realizadas,
las métricas empleadas para la planificación y sus estimaciones.

El proyecto se desarrollará siguiendo la metodología de trabajo ágil Scrum (ver
\cref{subsec:scrum}), una metodología que permite una planificación iterativa
e incremental, facilitando la adaptación a los cambios y la mejora continua del
proceso de desarrollo.

Esta metodología se adapta bien al proyecto, ya que permite que tanto el autor
como los tutores puedan llevar un seguimiento del avance del proyecto, y
modificar las tareas a realizar en función de los resultados obtenidos en cada
iteración.

Por ejemplo, si una tarea resulta más compleja de lo previsto inicialmente, se
podrá decidir si merece la pena dedicar más esfuerzo a esa tarea, o si es mejor
sustituirla por una alternativa más fácil de implementar, o incluso eliminarla
si no es esencial para el proyecto.

También evita la necesidad de una planificación detallada y exhaustiva al
inicio del proyecto, que hubiera sido imposible dada la variabilidad de la
cantidad de tiempo que el autor ha podido dedicar al proyecto (por su trabajo a
tiempo completo y cargas familiares), y la gran cantidad de incógnitas que
existían al comienzo del mismo.

# Scrum

Este proyecto se ha desarrollado siguiendo el marco de trabajo Scrum[^1]. A
continuación se presenta una breve introducción a Scrum, basada parcialmente en
la Guía de Scrum\cite{scrum-guide}, documento que define el marco de
trabajo Scrum y que se actualiza periódicamente.

También se explica, para cada una de las secciones, cómo se aplica esta parte
del marco teórico al desarrollo del proyecto, en la práctica.

![Diagrama gráfico del proceso Scrum (Fuente: Lakeworks)](images/scrum.pdf)

[^1]: Aunque lo parezca, Scrum no es un acrónimo, sino un término tomado del
rugby, que hace referencia a una formación de jugadores en el campo, en la que
cada uno de los jugadores tiene un rol específico, y todos trabajan juntos para
conseguir un objetivo común (el balón) \cite{scrum-meaning}

## Definición

Según los autores originales de Scrum, Schwaber y Sutherland \cite{scrum-guide},
\enquote{Scrum es un marco [de trabajo] ligero que ayuda a las personas,
equipos y organizaciones a generar valor a través de soluciones adaptables para
problemas complejos} (p. 3).

El enfoque del proceso Scrum es «iterativo e incremental», lo que significa que
se realizan entregas periódicas de un producto que va evolucionando a lo largo
del tiempo, y que se va adaptando a las necesidades del cliente.

## Roles

En Scrum, existen tres roles principales: el *Product Owner*, el *Scrum Master*
y el *Development Team*.

### Product Owner

El *Product Owner* o dueño del producto es el responsable de maximizar el valor
del producto resultante del trabajo del equipo Scrum. El dueño del producto es
quien define el objetivo del producto, y quien gestiona el *Product Backlog*.

### Scrum Master

El *Scrum Master* es el responsable de asegurar que Scrum se entiende y se
aplica correctamente. Actúa de intermediario entre el *Product Owner* y el
*Development Team*, facilitando la comunicación y la colaboración entre ambos,
y ayudando a resolver los obstáculos que puedan encontrar durante el proceso de
desarrollo.

Además, al delegársele las labores de seguimiento del flujo de trabajo, permite
que el equipo de desarrollo se centre en progresar hacia el objetivo del
producto.

### Development Team

El *Development Team* o equipo de desarrollo es el grupo de personas que se
encarga de desarrollar las funcionalidades de los incrementos del producto.

El equipo de desarrollo es autoorganizado, lo que significa que son ellos
quienes deciden cómo llevar a cabo el trabajo, y son responsables de la calidad
del producto que entregan. \cite{scrum-guide} (p. 7).

El equipo de desarrollo es multidisciplinar, y puede estar formado por
desarrolladores, diseñadores, testers, etc. El equipo tiene todas las
habilidades necesarias para generar los incrementos del producto.

Los miembros del equipo de desarrollo son iguales entre sí, y no hay jerarquías
ni equipos de trabajo especializados.

El tamaño del equipo de desarrollo debe ser lo suficientemente grande como para
completar el trabajo, pero lo suficientemente pequeño como para que la
comunicación y la colaboración sean efectivas. La Guía de Scrum recomienda un
tamaño de entre 3 y 9 personas \cite{scrum-guide} (p. 7)

## Aplicación al Trabajo de Fin de Grado

En el caso del Trabajo de Fin de Grado, los *Product Owners* son los tutores
del TFG, quienes definen el objetivo del producto y gestiona la pila
del producto.

El autor cumple los roles tanto de Scrum Master como el equipo de desarrollo,
asegurándose de cumplir con el marco de trabajo Scrum, hablando con los
*Product Owners* para definir el objetivo del producto y la pila del producto,
y desarrollando los incrementos del producto.

## Flujo de trabajo

El trabajo en Scrum se organiza en sprints, que son «eventos de longitud fija de
un mes o menos» \cite{scrum-guide} (p. 8), cuyo objetivo es entregar uno (o
más) incrementos del producto. Al finalizar un sprint, se comienza el
siguiente, volviendo a tener lugar todas las reuniones y eventos necesarios, y
así sucesivamente hasta que se alcance el objetivo del producto.

### Sprint Planning

Cada sprint comienza con una reunión de planificación, llamada *Sprint
Planning*, en la que los integrantes del equipo definen el trabajo a realizar
durante el sprint (confeccionando el *Sprint Backlog*) y establecen un objetivo
para el mismo.

### Daily Scrum

Tras la planificación, el equipo trabaja en las tareas definidas, y se reúne
diariamente en una reunión llamada *Daily Scrum*, cuyo propósito es
\enquote{inspeccionar el progreso hacia el Objetivo del Sprint y adaptar el
Sprint Backlog según sea necesario, ajustando el próximo trabajo planeado}
\cite{scrum-guide} (p. 9).

El *Daily Scrum* es una reunión breve, de máximo 15 minutos \cite{scrum-guide}
(p. 9), en la que los desarrolladores comunican su progreso, los obstáculos que
han encontrado y lo que planean hacer a continuación.

### Sprint Review

Durante la Sprint Review (en español, revisión del sprint), que ocurre al final
de cada sprint, como penúltimo evento, participan todos los roles de Scrum, y
se revisa el trabajo realizado durante el sprint, el equipo presenta el
incremento del producto.

Entonces se recoge la retroalimentación del dueño del producto o Product Owner,
y se discute el progreso hacia el objetivo del producto y qué hacer a continuación.

### Sprint Retrospective

Finalmente, tras la revisión del sprint, se realiza una reunión retrospectiva
del sprint o Sprint Retrospective, en la que el equipo reflexiona sobre el
trabajo realizado, y sobre qué cambios hacer para mejorar su eficacia y
eficiencia de cara al siguiente sprint.

En cada sprint se planifica el trabajo a realizar (*Sprint Planning*), se
trabaja y revisa el trabajo realizado (*Daily Scrum*) y, al final (pero aún
dentro del sprint), se revisa el trabajo realizado (*Sprint Review*) y se
reflexiona sobre el trabajo realizado y cómo mejorar para el siguiente sprint
(*Sprint Retrospective*).

### Aplicación al Trabajo de Fin de Grado

El caso del Trabajo de Fin de Grado es un caso particular, ya que el equipo de
desarrollo está formado por una sola persona, el autor. Por esto, no se han
realizado reuniones diarias (*Daily Scrum*).

Las reuniones de revisión del sprint (Sprint Review) se han llevado a cabo
semanalmente, y han consistido en una revisión del trabajo realizado durante la
semana, así como una revisión del progreso hacia el objetivo del producto, y
una discusión sobre qué hacer a continuación.

## Artefactos

Los artefactos de Scrum son los elementos que se utilizan para gestionar el
trabajo y el progreso del equipo, y son tres: el *Product Backlog*, el *Sprint
Backlog* y el *Incremento*. Cada uno de ellos se utiliza para conseguir llegar
a una meta específica, y se actualizan periódicamente a lo largo del proceso de
desarrollo.

### Product Backlog o pila del producto

La pila del producto \enquote{es una lista emergente y ordenada de lo que se
necesita para mejorar el producto} \cite{scrum-guide} (p. 11). Las tareas que
se incluyen en la pila del producto son las que, durante la planificación del
sprint, se seleccionan para formar parte del *Sprint Backlog*.

El objetivo del producto es un objetivo a largo plazo sobre el que se puede
planificar qué tareas incluir en la pila del producto. Surge de la interacción
con el dueño del producto o *Product Owner*.

### Sprint Backlog o pila del sprint

La pila del sprint \enquote{se compone del objetivo sprint (por qué), el
conjunto de elementos de trabajo pendiente de producto seleccionados para el
Sprint (qué), así como un plan accionable para entregar el incremento (cómo)}
\cite{scrum-guide} (p. 12).

Es decir, las tareas que se van a realizar durante el sprint, y que se
seleccionan de la pila del producto durante la planificación del sprint. La
desarrollan los desarrolladores, y se revisa (y reforma si fuese necesario)
durante el *Daily Scrum*.

### Increment o incremento del producto

Los incrementos son cada uno de los pasos (incrementales) que se dan para
alcanzar el objetivo del producto. Son \enquote{aditivo[s] a todos los incrementos
anteriores y verificado[s] a fondo, asegurando que todos los incrementos
funcionen juntos} \cite{scrum-guide} (p. 12).

Son, además, utilizables, lo que significa que se pueden entregar al cliente, y
que este puede utilizarlos para obtener valor del producto. Durante cada
sprint, como se ha visto anteriormente, se pueden entregar uno o varios
incrementos del producto.

### Aplicación al Trabajo de Fin de Grado

Los artefactos Scrum se han mantenido intactos al aplicar el marco teórico al
TFG.

## Estimaciones de tiempo

Antes de comenzar los sprint, el equipo de desarrollo debe estimar el tiempo
que tardará en completar cada tarea de la pila del producto. Esto se puede
hacer calculando el tiempo en horas que se tardará, directamente, o bien
mediante unidades de medida relativas, como puntos de historia (*story points*)
o tamaños de camiseta (*t-shirt sizes*).

Comenzando por los *story points*, según escribe Derek Davidson
\cite{why-story-points}, \enquote{un punto de historia es una unidad de medida
relativa, decidida y utilizada por equipos Scrum individuales, para
proporcionar estimaciones relativas del esfuerzo necesario para completar los
requisitos}.

El uso de puntos de historia es una práctica común en Scrum, y permite estimar
el esfuerzo necesario para completar una tarea de forma relativa, comparándola
con otras tareas, en lugar de absoluta.

La ventaja de utilizar puntos de historia en vez de una estimación clásica de
tiempo es que, según el coautor de la Guía de Scrum, Sutherland
\cite{story-points-vs-hours} \enquote{los puntos de historia dan estimaciones
más precisas, reducen el tiempo de planificación drásticamente, predicen las
fechas de lanzamiento con mayor precisión, y ayudan a
los equipos a mejorar su desempeño}.

Los tamaños de camiseta son otra forma de estimar el esfuerzo necesario para
completar una tarea. Son muy parecidos a los puntos de historia, pero en lugar
de utilizar una escala numérica, se utilizan tamaños de camiseta (XS, S, M, L,
XL, XXL) para representar el esfuerzo necesario para completar la tarea.

Esta forma de estimación es más intuitiva, y puede ser más fácil de entender
para personas que no están familiarizadas con Scrum o con la estimación de
tiempo en general. Sin embargo, puede ser menos precisa que los puntos de
historia, ya que no se puede comparar directamente el esfuerzo necesario para
completar una tarea con el de otra.

Además, resultan más complicados de utilizar a la hora de desarrollar gráficos,
ya que no se pueden utilizar directamente para calcular el trabajo pendiente o
el trabajo realizado, y requieren una conversión a puntos de historia para este
fin.

### Aplicación al Trabajo de Fin de Grado

Para el Trabajo de Fin de Grado, se han utilizado puntos de historia como
medida de estimación de esfuerzo, y no se ha realizado una contabilización
exhaustiva del tiempo empleado en cada tarea.

Esto se debe a que se considera que lo importante es el esfuerzo necesario para
completar una tarea, y no el tiempo que se tarda en completarla, siendo el
tiempo una medida complicada de estimar, y que puede variar mucho, ya no solo
dependiendo del esfuerzo invertido, sino también de factores externos como el
el tiempo disponible para trabajar en la tarea.

# Planificación temporal

TODO
