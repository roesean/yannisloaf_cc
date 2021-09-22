const config = {
	"triggers": {
		"bits": [       
			{
				"amount": "",
				"actions": ["config"],
				"time": "?base_time + amount" 
			} 
		],
		"subs": [
			{
				"amount": "config",
				"actions": ["config"],
				"time": "?base_time + amount",
				"community": "?true"
			} 
		],
		"donations": [
			{
				"amount": "config",
				"actions": ["config"],
				"time": "?base_time + amount" 
			},
			{
				"amount": "config",
				"actions": ["config"],
				"time": "?base_time + amount" 
			}
		]
	}
}

// User config
const userActions = {   
	"supported_actions": {
		"action_macro": {
			"keys": ["w_down,s_down", 60000, "w_up,s_up"],
		},
		"action_swap": {
			"map_from": ["w", "a" , "d", "s"],
			"map_to": ["s", "d" , "a", "w"],
			"randomize": "true?false"
		}
	}
}

// Out of the box
