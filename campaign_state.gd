extends Node

var vitorias: int = 0
var em_campanha: bool = false
var resultado_ultima_batalha: String = "none"
var campanha_concluida: bool = false

func iniciar_nova_campanha() -> void:
	vitorias = 0
	em_campanha = true
	resultado_ultima_batalha = "none"
	campanha_concluida = false

func registrar_vitoria() -> void:
	resultado_ultima_batalha = "vitoria"
	vitorias = min(vitorias + 1, 3)
	if vitorias >= 3:
		campanha_concluida = true

func registrar_derrota() -> void:
	resultado_ultima_batalha = "derrota"
	vitorias = 0
	em_campanha = false
	campanha_concluida = false

func encerrar_campanha() -> void:
	em_campanha = false
	resultado_ultima_batalha = "none"
