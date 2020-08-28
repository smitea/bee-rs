Const OWN_PROCESS = &H10
Const ERR_CONTROL = &H2
Const INTERACTIVE = False

Const NET_FW_IP_PROTOCOL_TCP = 6
Const NET_FW_IP_PROTOCOL_UDP = 17

Const NET_FW_ACTION_ALLOW = 1

Const NET_FW_PROFILE2_ALL = 2147483647

InstallPath = Session.Property("CustomActionData")
ServiceName = "BethunexAgent"
DisplayName = "Bethunex Agent"
ExePath = InstallPath & "hive.exe"
ServicePort = 6142

Set ObjWMI = GetObject("winmgmts:" & "{impersonationLevel=impersonate, (Security)}!\\.\root\cimv2")
Set ObjSvr = ObjWMI.Get("Win32_Service")
Result = ObjSvr.Create(ServiceName, DisplayName, ExePath, OWN_PROCESS, ERR_CONTROL, "Automatic", INTERACTIVE, "LocalSystem", "")

Set ObjService = ObjWMI.Get("Win32_Service.Name='" & ServiceName & "'")
Result = ObjService.StartService()

Set FwPolicy2 = CreateObject("HNetCfg.FwPolicy2")
Set RulesObject = FwPolicy2.Rules
Set NewRule = CreateObject("HNetCfg.FWRule")
NewRule.Name = ServiceName
NewRule.Applicationname = ExePath
NewRule.Servicename = ServiceName
NewRule.Profiles = NET_FW_PROFILE2_ALL
NewRule.Enabled = TRUE
NewRule.InterfaceTypes = "All"
NewRule.Action = NET_FW_ACTION_ALLOW      
RulesObject.Add NewRule