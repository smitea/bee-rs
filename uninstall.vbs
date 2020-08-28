ServiceName = "BethunexAgent"

Set ObjWMI = GetObject("winmgmts:" & "{impersonationLevel=impersonate, (Security)}!\\.\root\cimv2")
Set OService = ObjWMI.Get("Win32_Service.Name='" & ServiceName & "'")
OService.StopService()
OService.Delete()

Set fwPolicy2 = CreateObject("HNetCfg.FwPolicy2")
Set RulesObject = fwPolicy2.Rules
RulesObject.Remove ServiceName