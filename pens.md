## https://emias.info/api/emc/appointment-eip/v1/?getReferralsInfo
```json
{
    id: "1JbioYX9OAXea71AXSFy7",
    jsonrpc: "2.0",
    method: "getReferralsInfo",
    params:  {
        omsNumber: "7788899730000765", 
        birthDate: "2001-11-19"
    }
}
```

## https://emias.info/api/emc/appointment-eip/v1/?getDoctorsInfo
```json
{
    id: "lK3l04E4cDdZv8X10CPZG"
    jsonrpc: "2.0"
    method: "getDoctorsInfo"
    params: {
        omsNumber: "7788899730000765", 
        birthDate: "2001-11-19", 
        referralId: 172704541983
    }
}
```

## https://emias.info/api/emc/appointment-eip/v1/?getAvailableResourceScheduleInfo
```json
{
    id: "lK3l04E4cDdZv8X10CPZG"
    jsonrpc: "2.0"
    method: "getAvailableResourceScheduleInfo"
    params: {
        omsNumber: "7788899730000765", 
        birthDate: "2001-11-19", 
        availableResourceId: 19605506587,
        complexResourceId: 200992738,        
        referralId: 172704541983
    }
}
```