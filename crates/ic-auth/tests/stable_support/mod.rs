use ic_storage::{
    AllocationGrants, GenericAllocationPolicy, MemoryAuthority, MemoryRequest, MemoryRuntime,
    RuntimeMemory, SealedDeclarationSnapshot, StableKey, VectorMemory,
};

pub fn lease(backing: &VectorMemory) -> RuntimeMemory<VectorMemory> {
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    let name = "auth.test.v1";
    let requests =
        SealedDeclarationSnapshot::new(vec![MemoryRequest::new("auth", name).unwrap()]).unwrap();
    let grants =
        AllocationGrants::new(vec![MemoryAuthority::new("auth", "auth.").unwrap()]).unwrap();
    host.bootstrap(&requests, &grants, &GenericAllocationPolicy)
        .unwrap();
    host.open_memory(&StableKey::parse(name).unwrap()).unwrap()
}
