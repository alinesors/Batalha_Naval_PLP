extends Control

const SHIP_SCALE := Vector2(0.085, 0.085)
const MOVE_DURATION := 1.0
const MARKER_OFFSET_FACIL := Vector2(-20.0, 25.0)
const MARKER_OFFSET_INTERMEDIARIA := Vector2(-37.5, 25.0)
const MARKER_OFFSET_DIFICIL := Vector2(-22.5, 25.0)
const PATH_COLOR := Color(0.862745, 0.25098, 0.25098, 0.95)
const PATH_WIDTH := 3.0
const DASH_LENGTH := 9.0
const GAP_LENGTH := 7.0

@onready var progress_line: Line2D = $ProgressLine
@onready var ship_sprite: Sprite2D = $Ship
@onready var marker_facil: Label = $IAFacil
@onready var marker_intermediaria: Label = $IAIntermediaria
@onready var marker_dificil: Label = $IADificil
@onready var hud_hint: Label = $HUD/Hint

var stage_positions: Array[Vector2] = []
var path_points: PackedVector2Array = PackedVector2Array()
var current_stage := 0
var active_tween: Tween

func _ready() -> void:
	ship_sprite.scale = SHIP_SCALE
	progress_line.visible = false
	get_viewport().size_changed.connect(_on_viewport_size_changed)
	_update_layout()
	_set_stage(0, false)
	hud_hint.text = "Mock campanha: 0=inicio | 1=IA facil | 2=IA intermediaria | 3=IA dificil | ESC=voltar"

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_0:
				set_stage_0()
			KEY_1:
				set_stage_1()
			KEY_2:
				set_stage_2()
			KEY_3:
				set_stage_3()
			KEY_ESCAPE:
				get_tree().change_scene_to_file("res://MenuPrincipal.tscn")

func _on_viewport_size_changed() -> void:
	var previous_position := ship_sprite.global_position
	var previous_stage := current_stage
	_update_layout()

	# Preserva o estágio atual após resize; fallback pela posição anterior.
	if previous_stage >= 0 and previous_stage < stage_positions.size():
		ship_sprite.global_position = stage_positions[previous_stage]
	else:
		ship_sprite.global_position = previous_position

func _update_layout() -> void:
	var size := get_viewport_rect().size
	var x_center := size.x * 0.52

	var start := Vector2(x_center, size.y * 0.83)
	var facil := Vector2(x_center, size.y * 0.66)
	var intermediaria := Vector2(x_center, size.y * 0.48)
	var dificil := Vector2(x_center, size.y * 0.30)

	stage_positions = [start, facil, intermediaria, dificil]

	marker_facil.position = facil + MARKER_OFFSET_FACIL
	marker_intermediaria.position = intermediaria + MARKER_OFFSET_INTERMEDIARIA
	marker_dificil.position = dificil + MARKER_OFFSET_DIFICIL

	var curved_points := PackedVector2Array()
	var segment_dificil_intermediaria := _build_curved_segment(dificil, intermediaria, 35.0)
	var segment_intermediaria_facil := _build_curved_segment(intermediaria, facil, -42.5)
	var segment_facil_inicio := _build_curved_segment(facil, start, 32.5)

	curved_points.append_array(segment_dificil_intermediaria)

	for i in range(1, segment_intermediaria_facil.size()):
		curved_points.append(segment_intermediaria_facil[i])

	for i in range(1, segment_facil_inicio.size()):
		curved_points.append(segment_facil_inicio[i])

	path_points = curved_points
	queue_redraw()

func _build_curved_segment(from_point: Vector2, to_point: Vector2, bend: float, samples: int = 8) -> PackedVector2Array:
	var mid := from_point.lerp(to_point, 0.5)
	var control := mid + Vector2(bend, 0.0)
	var points := PackedVector2Array()

	for i in range(samples + 1):
		var t := float(i) / float(samples)
		var inv_t := 1.0 - t
		var point := inv_t * inv_t * from_point + 2.0 * inv_t * t * control + t * t * to_point
		points.append(point)

	return points

func _draw() -> void:
	if path_points.size() < 2:
		return

	_draw_dashed_polyline(path_points, PATH_COLOR, PATH_WIDTH, DASH_LENGTH, GAP_LENGTH)

func _draw_dashed_polyline(points: PackedVector2Array, color: Color, width: float, dash_length: float, gap_length: float) -> void:
	var drawing_dash: bool = true
	var progress_in_phase: float = 0.0

	for i in range(points.size() - 1):
		var segment_start: Vector2 = points[i]
		var segment_end: Vector2 = points[i + 1]
		var segment_vector: Vector2 = segment_end - segment_start
		var segment_length: float = segment_vector.length()

		if segment_length <= 0.0:
			continue

		var direction: Vector2 = segment_vector / segment_length
		var traveled: float = 0.0

		while traveled < segment_length:
			var current_phase_length: float = dash_length if drawing_dash else gap_length
			var remaining_phase: float = current_phase_length - progress_in_phase
			var remaining_segment: float = segment_length - traveled
			var step: float = min(remaining_phase, remaining_segment)

			if drawing_dash and step > 0.0:
				var from_point: Vector2 = segment_start + direction * traveled
				var to_point: Vector2 = segment_start + direction * (traveled + step)
				draw_line(from_point, to_point, color, width, true)

			traveled += step
			progress_in_phase += step

			if progress_in_phase >= current_phase_length:
				progress_in_phase = 0.0
				drawing_dash = not drawing_dash

func _set_stage(stage: int, animate: bool) -> void:
	if stage < 0 or stage >= stage_positions.size():
		return

	current_stage = stage
	var target_position := stage_positions[stage]

	if is_instance_valid(active_tween):
		active_tween.kill()

	if animate:
		active_tween = create_tween()
		active_tween.set_trans(Tween.TRANS_SINE)
		active_tween.set_ease(Tween.EASE_IN_OUT)
		active_tween.tween_property(ship_sprite, "global_position", target_position, MOVE_DURATION)
	else:
		ship_sprite.global_position = target_position

func set_stage_0() -> void:
	_set_stage(0, true)

func set_stage_1() -> void:
	_set_stage(1, true)

func set_stage_2() -> void:
	_set_stage(2, true)

func set_stage_3() -> void:
	_set_stage(3, true)
