use super::snapshot::snapshot_ty;
use super::utils::reinterpret_cast_signature;
use crate::define_libfunc_hierarchy;
use crate::extensions::lib_func::{
    DeferredOutputKind, LibfuncSignature, OutputVarInfo, SierraApChange,
    SignatureAndTypeGenericLibfunc, SignatureSpecializationContext,
    WrapSignatureAndTypeGenericLibfunc,
};
use crate::extensions::type_specialization_context::TypeSpecializationContext;
use crate::extensions::types::{
    GenericTypeArgGenericType, GenericTypeArgGenericTypeWrapper, TypeInfo,
};
use crate::extensions::{NamedType, OutputVarReferenceInfo, SpecializationError};
use crate::ids::{ConcreteTypeId, GenericTypeId};

/// Type wrapping a value.
#[derive(Default)]
pub struct BoxTypeWrapped {}
impl GenericTypeArgGenericType for BoxTypeWrapped {
    const ID: GenericTypeId = GenericTypeId::new_inline("Box");

    fn calc_info(
        &self,
        _context: &dyn TypeSpecializationContext,
        long_id: crate::program::ConcreteTypeLongId,
        TypeInfo { storable, droppable, duplicatable, .. }: TypeInfo,
    ) -> Result<TypeInfo, SpecializationError> {
        if storable {
            Ok(TypeInfo { long_id, zero_sized: false, storable, droppable, duplicatable })
        } else {
            Err(SpecializationError::UnsupportedGenericArg)
        }
    }
}
pub type BoxType = GenericTypeArgGenericTypeWrapper<BoxTypeWrapped>;

define_libfunc_hierarchy! {
    pub enum BoxLibfunc {
        Into(IntoBoxLibfunc),
        Unbox(UnboxLibfunc),
        ForwardSnapshot(BoxForwardSnapshotLibfunc),
    }, BoxConcreteLibfunc
}

/// Libfunc for wrapping an object of type T into a box.
#[derive(Default)]
pub struct IntoBoxLibfuncWrapped {}
impl SignatureAndTypeGenericLibfunc for IntoBoxLibfuncWrapped {
    const STR_ID: &'static str = "into_box";

    fn specialize_signature(
        &self,
        context: &dyn SignatureSpecializationContext,
        ty: ConcreteTypeId,
    ) -> Result<LibfuncSignature, SpecializationError> {
        Ok(LibfuncSignature::new_non_branch(
            vec![ty.clone()],
            vec![OutputVarInfo {
                ty: context.get_wrapped_concrete_type(BoxType::id(), ty)?,
                ref_info: OutputVarReferenceInfo::NewTempVar { idx: 0 },
            }],
            SierraApChange::Known { new_vars_only: true },
        ))
    }
}
pub type IntoBoxLibfunc = WrapSignatureAndTypeGenericLibfunc<IntoBoxLibfuncWrapped>;

/// Libfunc for unboxing a `Box<T>` back into a T.
#[derive(Default)]
pub struct UnboxLibfuncWrapped {}
impl SignatureAndTypeGenericLibfunc for UnboxLibfuncWrapped {
    const STR_ID: &'static str = "unbox";

    fn specialize_signature(
        &self,
        context: &dyn SignatureSpecializationContext,
        ty: ConcreteTypeId,
    ) -> Result<LibfuncSignature, SpecializationError> {
        Ok(LibfuncSignature::new_non_branch(
            vec![context.get_wrapped_concrete_type(BoxType::id(), ty.clone())?],
            vec![OutputVarInfo {
                ty: ty.clone(),
                ref_info: if context.get_type_info(ty)?.zero_sized {
                    OutputVarReferenceInfo::ZeroSized
                } else {
                    OutputVarReferenceInfo::Deferred(DeferredOutputKind::Generic)
                },
            }],
            SierraApChange::Known { new_vars_only: true },
        ))
    }
}
pub type UnboxLibfunc = WrapSignatureAndTypeGenericLibfunc<UnboxLibfuncWrapped>;

/// Libfunc for converting `@Box<T>` into `Box<@T>`.
#[derive(Default)]
pub struct BoxForwardSnapshotLibfuncWrapped {}
impl SignatureAndTypeGenericLibfunc for BoxForwardSnapshotLibfuncWrapped {
    const STR_ID: &'static str = "box_forward_snapshot";
    fn specialize_signature(
        &self,
        context: &dyn SignatureSpecializationContext,
        ty: ConcreteTypeId,
    ) -> Result<LibfuncSignature, SpecializationError> {
        Ok(reinterpret_cast_signature(
            snapshot_ty(context, context.get_wrapped_concrete_type(BoxType::id(), ty.clone())?)?,
            context.get_wrapped_concrete_type(BoxType::id(), snapshot_ty(context, ty)?)?,
        ))
    }
}
pub type BoxForwardSnapshotLibfunc =
    WrapSignatureAndTypeGenericLibfunc<BoxForwardSnapshotLibfuncWrapped>;
