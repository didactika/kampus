---
id: DEC-012
status: accepted
---

# DEC-012 Kampus terminology

- Date: 2026-11-17
- Context: The owner wants Kampus to have its own vocabulary, clear to learners and staff, without losing the concept behind each term. The importer, the converter and the migration guide need a stable vocabulary to map to.
- Options considered:
  - Use the most common names of the sector for everything.
  - Invent new names for everything.
  - Choose a name per concept, keeping the well-known ones (chosen; the owner reviewed the list term by term on 2026-11-17).
- Decision: the glossary below is the vocabulary of Kampus in the specification, code, API and handbook.
  - Every term is a default: campuses rename them through the terminology layer of the customisation framework (DEC-008).
  - The mapping from other platforms' vocabularies lives with the importer and `moodle2kampus` (DEC-006), not in the specification.

  | Term | Spanish | Concept |
  | --- | --- | --- |
  | Campus | Campus | The whole installation, or one tenant of it |
  | Control centre | Centro de control | Where the platform is configured |
  | Home | Inicio | The personal landing page |
  | Course category | Categoría de cursos | Nested container of courses |
  | Course | Curso | A course |
  | Course layout | Diseño de curso | How sections and course modules are presented |
  | Section | Sección | A part of a course; also a permission scope (DEC-007) |
  | Course module | Módulo de curso | Anything placed in a section |
  | Activity | Actividad | A course module learners do |
  | Material | Material | A course module learners read or watch |
  | Widget | Widget | A side panel component |
  | Scope | Ámbito | Where a permission or setting applies |
  | Permission | Permiso | One thing a role allows |
  | Permission override | Ajuste de permiso | A change to a role in one scope |
  | Exception | Excepción | Other dates, attempts or time for one user or group, available in every activity (RF-054) |
  | Enrolment | Matriculación | A person's participation in a course |
  | Enrolment method | Método de matriculación | How people join a course |
  | Group | Grupo | A set of people inside a course |
  | Group set | Conjunto de grupos | A set of groups |
  | System group | Grupo del sistema | Campus- or category-wide set of people |
  | Academic records | Expediente académico | Grades of a course and how they add up |
  | Grade record | Registro de calificación | One graded thing |
  | Progress | Progreso | Tracking what is finished |
  | Achievement | Logro | A recognition earned by a learner |
  | Skill | Habilidad | What a learner can demonstrate |
  | Assignment | Tarea | Work submitted for assessment |
  | Quiz | Cuestionario | Questions answered and graded |
  | Question bank | Banco de preguntas | Reusable questions |
  | Forum | Foro | Threaded conversation |
  | Choice | Consulta | One question, chosen answers |
  | Survey | Encuesta | A questionnaire |
  | Non-editing teacher | Profesor sin permiso de edición | Teaches without changing the course |
  | Mentor | Mentor | Follows assigned learners |
  | Job | Tarea programada | Work run in the background |
  | Event listener | Escucha de eventos | Code that reacts to an event |
  | API endpoint | Endpoint de la API | A call of the public API |
  | Course backup (`.kpk`) | Copia de seguridad del curso | A portable copy of a course, in Kampus format |
  | Plugin | Plugin | An installable addition (DEC-003) |
  | Theme | Tema | Built from design tokens (DEC-013) |

- Consequences:
  - The specification is written in this vocabulary.
  - The converter (DEC-006), the importer and the migration guide translate other vocabularies into this one.
  - Crates follow the vocabulary: `kampus-records` (academic records) and `kampus-progress` (progress).
- Affected artifacts: `docs/`, the handbook, `moodle2kampus`, the importer.
