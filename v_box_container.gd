extends VBoxContainer

func _ready():
	$start.grab_focus()


func _on_start_pressed():
	print("pressionou start")


func _unhandled_input(event):
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_C:
			CampaignState.iniciar_nova_campanha()
			get_tree().change_scene_to_file("res://scenes/modo_campanha.tscn")
